//! Bridge USB Serial/JTAG para o TokenBuddy (ESP32-C6).
//!
//! Protocolo: JSON linha-a-linha com campo `v:1`; comandos aguardam ACK por `id`.
//!
//! Design: um único worker multiplexa leitura, ACKs, fila de comandos e
//! heartbeat/telemetria em um loop. Requests chegam por um canal; a resposta
//! volta por um par de canais criado por chamada. Não há lock contido durante
//! I/O — o worker só trava o estado para atualizar snapshot.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum BridgeError {
    Disconnected,
    QueueFull,
    Timeout,
    Command(String),
    Io(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Dispositivo USB desconectado ou identidade nao confirmada"),
            Self::QueueFull => write!(f, "Fila de comandos cheia"),
            Self::Timeout => write!(f, "Dispositivo nao confirmou o comando"),
            Self::Command(m) => f.write_str(m),
            Self::Io(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for BridgeError {}

/// Abstração mínima sobre a porta serial (permite fake em testes).
pub trait SerialPort: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error>;
    fn write_all(&mut self, data: &[u8]) -> Result<(), std::io::Error>;
    fn close(&mut self) -> Result<(), std::io::Error>;
}

struct RealSerial {
    inner: Box<dyn serialport::SerialPort>,
}

impl RealSerial {
    fn open(port: &str) -> Result<Self, BridgeError> {
        let s = serialport::new(port, 115200u32)
            .open()
            .map_err(|e| BridgeError::Io(e.to_string()))?;
        Ok(Self { inner: s })
    }
}

impl SerialPort for RealSerial {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.inner.read(buf)
    }
    fn write_all(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.inner.write_all(data)
    }
    fn close(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub struct Bridge {
    pub port: String,
    expected_mac: String,
    tx: mpsc::Sender<(Value, mpsc::Sender<Result<Value, BridgeError>>)>,
    state: Arc<Mutex<State>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

#[derive(Default)]
struct State {
    connected: bool,
    device: Value,
    telemetry: Value,
    last_received: Option<Instant>,
    last_error: Option<String>,
    errors: Vec<Value>,
    last_agent: Option<Value>,
}

impl Bridge {
    pub fn new(port: &str, expected_mac: &str) -> Self {
        let (tx, _rx) = mpsc::channel();
        Self {
            port: port.to_string(),
            expected_mac: expected_mac.to_lowercase(),
            tx,
            state: Arc::new(Mutex::new(State::default())),
            worker: None,
        }
    }

    /// Inicia o worker. `metrics` é chamado a cada ~2s para produzir telemetria.
    pub fn start<F>(&mut self, metrics: F)
    where
        F: FnMut() -> Value + Send + 'static,
    {
        let port = self.port.clone();
        let expected_mac = self.expected_mac.clone();
        let state = Arc::clone(&self.state);
        let (tx, rx) = mpsc::channel::<(Value, mpsc::Sender<Result<Value, BridgeError>>)>();
        self.tx = tx;
        self.worker = Some(std::thread::spawn(move || {
            worker_loop(port, expected_mac, rx, state, metrics);
        }));
    }

    pub fn stop(&mut self) {
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
        let mut g = self.state.lock().unwrap();
        g.connected = false;
    }

    pub fn snapshot(&self) -> Value {
        let g = self.state.lock().unwrap();
        serde_json::json!({
            "serial_connected": g.connected,
            "device_fresh": g.connected && g.last_received.map_or(false, |t| t.elapsed() < Duration::from_secs(3)),
            "device": g.device,
            "telemetry": g.telemetry,
            "last_error": g.last_error,
            "last_agent_event": g.last_agent,
            "recent_errors": g.errors,
            "queued": 0,
        })
    }

    /// Envia um comando e aguarda o ACK (timeout de 4s, como no Python).
    pub fn request(&self, message: Value) -> Result<Value, BridgeError> {
        let (rtx, rrx) = mpsc::channel();
        self.tx
            .send((message, rtx))
            .map_err(|_| BridgeError::QueueFull)?;
        match rrx.recv_timeout(Duration::from_secs(4)) {
            Ok(r) => r,
            Err(_) => Err(BridgeError::Timeout),
        }
    }
}

// ---------------------------------------------------------------------------
// Worker — loop de multiplexação
// ---------------------------------------------------------------------------

fn worker_loop<F>(
    port: String,
    expected_mac: String,
    rx: mpsc::Receiver<(Value, mpsc::Sender<Result<Value, BridgeError>>)>,
    state: Arc<Mutex<State>>,
    mut metrics: F,
) where
    F: FnMut() -> Value + Send + 'static,
{
    let mut serial: Option<Box<dyn SerialPort>> = None;
    let mut rxBuf = Vec::new();
    let mut next_id: u32 = 1;
    let mut heartbeat_at: Option<Instant> = None;
    let mut metrics_at: Option<Instant> = None;

    loop {
        // Conecta se necessário (valida VID/PID, fw e MAC; envia \n + heartbeat).
        if serial.is_none() {
            match connect(&port, &expected_mac, &mut rxBuf, &mut next_id, &state) {
                Ok(s) => {
                    serial = Some(s);
                    heartbeat_at = None;
                    metrics_at = None;
                }
                Err(e) => {
                    let mut g = state.lock().unwrap();
                    g.connected = false;
                    g.last_error = Some(e.to_string());
                    push_error(&mut g);
                    drop(g);
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        }

        let now = Instant::now();

        // Heartbeat a cada ~1s.
        if heartbeat_at.map_or(true, |t| now - t >= Duration::from_secs(1)) {
            match send_command(serial.as_mut().unwrap(), &mut rxBuf, &mut next_id, &serde_json::json!({"type": "heartbeat"})) {
                Ok(_) => { heartbeat_at = Some(now); }
                Err(e) => { on_error(&state, e); break; }
            }
        }

        // Telemetria a cada ~2s.
        if metrics_at.map_or(true, |t| now - t >= Duration::from_secs(2)) {
            let m = metrics();
            match send_command(serial.as_mut().unwrap(), &mut rxBuf, &mut next_id, &m) {
                Ok(_) => {
                    let mut g = state.lock().unwrap();
                    g.telemetry = m;
                    metrics_at = Some(now);
                }
                Err(e) => { on_error(&state, e); break; }
            }
        }

        // Processa requests um por vez (FIFO), como no Python.
        loop {
            let item = match rx.try_recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            let (message, reply) = item;
            let deadline = Instant::now() + Duration::from_secs(3);
            // Regra de uma tarefa ativa por vez (eventos de agente).
            if message.get("type").and_then(|x| x.as_str()) == Some("event") {
                let active = state.lock().unwrap().last_agent.clone();
                if let Some(ref a) = active {
                    if a.get("event").and_then(|x| x.as_str()) == Some("agent.started") {
                        if message.get("agent") != a.get("agent")
                            || message.get("task_id") != a.get("task_id")
                        {
                            let _ = reply.send(Err(BridgeError::Command(
                                "Outra tarefa esta ativa; MVP acompanha uma tarefa por vez".into(),
                            )));
                            continue;
                        }
                    }
                }
            }
            if Instant::now() > deadline {
                let _ = reply.send(Err(BridgeError::Timeout));
                continue;
            }
            match send_command(serial.as_mut().unwrap(), &mut rxBuf, &mut next_id, &message) {
                Ok(ack) => {
                    if message.get("type").and_then(|x| x.as_str()) == Some("event") {
                        let mut g = state.lock().unwrap();
                        g.last_agent = Some(message.clone());
                    }
                    let _ = reply.send(Ok(ack));
                }
                Err(e) => {
                    let _ = reply.send(Err(e.clone()));
                    // Erros de transporte derrubam a conexão; CommandError (rejeição
                    // do dispositivo) só falha o request, como no Python.
                    match &e {
                        BridgeError::Command(_) => {}
                        _ => { on_error(&state, e); break; }
                    }
                }
            }
        }

        // Lê e processa status (ACKs são consumidos dentro de send_command).
        if let Err(e) = pump(serial.as_mut().unwrap(), &mut rxBuf, &expected_mac, &state) {
            on_error(&state, e);
            break;
        }

        std::thread::sleep(Duration::from_millis(5));
    }

    if let Some(s) = serial.as_mut() {
        let _ = s.close();
    }
    let mut g = state.lock().unwrap();
    g.connected = false;
}

/// Lê da serial e atualiza `device` (status). ACKs são consumidos por send_command.
fn pump(
    serial: &mut Box<dyn SerialPort>,
    rxBuf: &mut Vec<u8>,
    expected_mac: &str,
    state: &Mutex<State>,
) -> Result<(), BridgeError> {
    let mut buf = [0u8; 256];
    loop {
        match serial.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => rxBuf.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
        if rxBuf.len() > 8192 {
            rxBuf.clear();
        }
    }
    while let Some(found) = rxBuf.iter().position(|&b| b == b'\n') {
        let end = found + 1;
        let line: Vec<u8> = rxBuf.drain(..end).collect();
        if let Ok(s) = std::str::from_utf8(&line[..line.len().saturating_sub(1)]) {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                process_line(v, expected_mac, state)?;
            }
        }
    }
    Ok(())
}

fn process_line(v: Value, expected_mac: &str, state: &Mutex<State>) -> Result<(), BridgeError> {
    if v.get("v").and_then(|x| x.as_u64()) != Some(1) {
        return Ok(());
    }
    // Status do dispositivo.
    if v.get("type").and_then(|x| x.as_str()) == Some("status")
        && v.get("device").and_then(|x| x.as_str()) == Some("TokenBuddy")
    {
        let mac = match v.get("mac").and_then(|x| x.as_str()) {
            Some(m) => m,
            None => return Ok(()),
        };
        if mac.to_lowercase() != expected_mac {
            return Err(BridgeError::Command(
                "MAC do dispositivo diferente da configurada".into(),
            ));
        }
        let mut g = state.lock().unwrap();
        g.device = v.clone();
        g.last_received = Some(Instant::now());
    }
    Ok(())
}

/// Envia um comando e aguarda o ACK (bloqueante, usado para heartbeat/telemetria).
fn send_command(
    serial: &mut Box<dyn SerialPort>,
    rxBuf: &mut Vec<u8>,
    next_id: &mut u32,
    message: &Value,
) -> Result<Value, BridgeError> {
    let id = *next_id;
    *next_id = if *next_id >= 2_147_483_647 { 1 } else { *next_id + 1 };
    let mut wire = message.clone();
    wire["v"] = serde_json::json!(1);
    wire["id"] = serde_json::json!(id);
    let bytes = serde_json::to_string(&wire).unwrap().into_bytes();
    if bytes.len() > 768 {
        return Err(BridgeError::Command("Comando excede 768 bytes".into()));
    }
    serial.write_all(&bytes).map_err(|e| BridgeError::Io(e.to_string()))?;
    serial.write_all(b"\n").map_err(|e| BridgeError::Io(e.to_string()))?;
    let deadline = Instant::now() + Duration::from_millis(1500);
    while Instant::now() < deadline {
        drain(serial, rxBuf)?;
        while let Some(found) = rxBuf.iter().position(|&b| b == b'\n') {
            let end = found + 1;
            let line: Vec<u8> = rxBuf.drain(..end).collect();
            if let Ok(s) = std::str::from_utf8(&line[..line.len().saturating_sub(1)]) {
                if let Ok(v) = serde_json::from_str::<Value>(s) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("ack")
                        && v.get("id").and_then(|x| x.as_u64()) == Some(id as u64)
                    {
                        if v.get("ok").and_then(|x| x.as_bool()) != Some(true) {
                            return Err(BridgeError::Command(
                                v.get("error")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("device_rejected")
                                    .to_string(),
                            ));
                        }
                        return Ok(v);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err(BridgeError::Timeout)
}

fn drain(serial: &mut Box<dyn SerialPort>, rxBuf: &mut Vec<u8>) -> Result<(), BridgeError> {
    let mut buf = [0u8; 256];
    loop {
        match serial.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => rxBuf.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
        if rxBuf.len() > 8192 {
            rxBuf.clear();
        }
    }
    Ok(())
}

fn on_error(state: &Arc<Mutex<State>>, e: BridgeError) {
    let mut g = state.lock().unwrap();
    g.connected = false;
    g.last_error = Some(e.to_string());
    push_error(&mut g);
}

fn push_error(g: &mut State) {
    if let Some(ref e) = g.last_error {
        g.errors.push(serde_json::json!({
            "at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            "error": e,
        }));
        if g.errors.len() > 16 {
            g.errors.drain(..g.errors.len() - 16);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serial fake: devolve cada chunk em uma única leitura (simula serial real); grava tudo num buffer.
    struct FakeSerial {
        chunks: std::sync::Mutex<std::vec::IntoIter<Vec<u8>>>,
        written: std::sync::Mutex<Vec<u8>>,
    }

    impl FakeSerial {
        fn new(chunks: &[&[u8]]) -> Self {
            Self {
                chunks: std::sync::Mutex::new(
                    chunks.iter().map(|c| c.to_vec()).collect::<Vec<_>>().into_iter(),
                ),
                written: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl SerialPort for FakeSerial {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
            let chunk = self.chunks.lock().unwrap().next();
            match chunk {
                Some(c) => {
                    let n = c.len().min(buf.len());
                    buf[..n].copy_from_slice(&c[..n]);
                    Ok(n)
                }
                None => Ok(0),
            }
        }
        fn write_all(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
            self.written.lock().unwrap().extend_from_slice(data);
            Ok(())
        }
        fn close(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[test]
    fn partial_lines_and_identity() {
        let state = Arc::new(Mutex::new(State::default()));
        let message = serde_json::json!({
            "v": 1, "type": "status", "device": "TokenBuddy", "mac": "aa:bb:cc:dd:ee:ff"
        });
        let wire = serde_json::to_string(&message).unwrap().into_bytes();
        // Primeiro chunk: linha de log + JSON inválido + status truncado.
        let mut first = b"log\nnot-json\n".to_vec();
        first.extend_from_slice(&wire[..10]);
        // Segundo chunk: resto da linha + newline para completar o frame.
        let mut second = wire[10..].to_vec();
        second.push(b'\n');

        let serial = FakeSerial::new(&[&first, &second]);
        let mut rxBuf = Vec::new();
        let mut s: Box<dyn SerialPort> = Box::new(serial);

        // Único pump: lê os dois chunks e processa as linhas completas.
        pump(&mut s, &mut rxBuf, "aa:bb:cc:dd:ee:ff", &state).unwrap();
        assert_eq!(state.lock().unwrap().device["mac"], "aa:bb:cc:dd:ee:ff");

        // MAC errado derruba a conexão.
        let bad = serde_json::json!({
            "v": 1, "type": "status", "device": "TokenBuddy", "mac": "11:22:33:44:55:66"
        });
        let mut bad_wire = serde_json::to_string(&bad).unwrap().into_bytes();
        bad_wire.push(b'\n');
        let serial3 = FakeSerial::new(&[&bad_wire]);
        let mut rxBuf2 = Vec::new();
        let mut s3: Box<dyn SerialPort> = Box::new(serial3);
        let err = pump(&mut s3, &mut rxBuf2, "aa:bb:cc:dd:ee:ff", &state).unwrap_err();
        assert!(matches!(err, BridgeError::Command(_)));
    }

    #[test]
    fn request_requires_connection() {
        let bridge = Bridge::new("TEST", "expected");
        // Worker nunca iniciado: tx sem receptor → request deve falhar.
        let err = bridge.request(serde_json::json!({"type": "heartbeat"})).unwrap_err();
        assert!(matches!(err, BridgeError::QueueFull));
    }

    #[test]
    fn send_command_matches_ack_by_id() {
        let status = serde_json::json!({
            "v": 1, "type": "status", "device": "TokenBuddy", "mac": "aa:bb:cc:dd:ee:ff"
        });
        // Status chega antes do ACK (id 1) — deve ser ignorado pelo send_command.
        let mut chunk = serde_json::to_string(&status).unwrap().into_bytes();
        chunk.extend_from_slice(b"\n");
        chunk.extend_from_slice(br#"{"v":1,"type":"ack","id":1,"ok":true}"#.as_ref());
        chunk.extend_from_slice(b"\n");

        let serial = FakeSerial::new(&[&chunk]);
        let mut rxBuf = Vec::new();
        let mut next_id = 1u32;
        let mut s: Box<dyn SerialPort> = Box::new(serial);
        let ack = send_command(
            &mut s,
            &mut rxBuf,
            &mut next_id,
            &serde_json::json!({"type": "heartbeat"}),
        )
        .unwrap();
        assert_eq!(ack["id"], 1);
        assert_eq!(next_id, 2); // id incrementa e não vaza para o próximo
    }

    #[test]
    fn send_command_rejects_device_error() {
        let mut chunk = br#"{"v":1,"type":"ack","id":1,"ok":false,"error":"priority"}"#.to_vec();
        chunk.extend_from_slice(b"\n");
        let serial = FakeSerial::new(&[&chunk]);
        let mut rxBuf = Vec::new();
        let mut next_id = 1u32;
        let mut s: Box<dyn SerialPort> = Box::new(serial);
        let err = send_command(
            &mut s,
            &mut rxBuf,
            &mut next_id,
            &serde_json::json!({"type": "settings"}),
        )
        .unwrap_err();
        assert!(matches!(err, BridgeError::Command(m) if m == "priority"));
    }

    #[test]
    fn send_command_refuses_oversized_wire() {
        let big = serde_json::json!({"type": "event", "payload": "x".repeat(800)});
        let serial = FakeSerial::new(&[]);
        let mut rxBuf = Vec::new();
        let mut next_id = 1u32;
        let mut s: Box<dyn SerialPort> = Box::new(serial);
        let err = send_command(&mut s, &mut rxBuf, &mut next_id, &big).unwrap_err();
        assert!(matches!(err, BridgeError::Command(m) if m.contains("768")));
    }
}

fn connect(
    port: &str,
    expected_mac: &str,
    rxBuf: &mut Vec<u8>,
    next_id: &mut u32,
    state: &Mutex<State>,
) -> Result<Box<dyn SerialPort>, BridgeError> {
    let ports = serialport::available_ports().map_err(|e| BridgeError::Io(e.to_string()))?;
    let candidate = ports.iter().find(|p| p.port_name.as_str() == port);
    let (vid, pid) = match candidate {
        Some(p) => match &p.port_type {
            serialport::SerialPortType::UsbPort(usb) => (usb.vid, usb.pid),
            _ => return Err(BridgeError::Disconnected),
        },
        None => return Err(BridgeError::Disconnected),
    };
    if (vid, pid) != (0x303A, 0x1001) {
        return Err(BridgeError::Command(
            "Porta configurada nao e Espressif USB Serial/JTAG".into(),
        ));
    }
    let mut serial: Box<dyn SerialPort> = Box::new(RealSerial::open(port)?);
    rxBuf.clear();
    let started = Instant::now();
    // Aguarda até 6s por um status com fw 0.3.0 e MAC esperado.
    while started.elapsed() < Duration::from_secs(6) {
        pump(&mut serial, rxBuf, expected_mac, state)?;
        let g = state.lock().unwrap();
        // last_received deve ter sido atualizado durante esta tentativa de conexão.
        let identified = g.last_received.map_or(false, |t| t.elapsed() <= started.elapsed())
            && g.device.get("fw") == Some(&serde_json::json!("0.3.0"));
        drop(g);
        if identified {
            // Encerra frame parcial deixado por um host anterior e confirma com heartbeat.
            serial.write_all(b"\n").map_err(|e| BridgeError::Io(e.to_string()))?;
            send_command(&mut serial, rxBuf, next_id, &serde_json::json!({"type": "heartbeat"}))?;
            let mut g = state.lock().unwrap();
            g.connected = true;
            g.last_error = None;
            let agent = g.last_agent.clone();
            drop(g);
            // Reenvia o evento de agente ativo, se houver.
            if let Some(ref a) = agent {
                if a.get("event").and_then(|x| x.as_str()) == Some("agent.started") {
                    send_command(&mut serial, rxBuf, next_id, a)?;
                }
            }
            return Ok(serial);
        }
    }
    Err(BridgeError::Command(
        "Firmware 0.3.0 / MAC esperado nao confirmado".into(),
    ))
}
