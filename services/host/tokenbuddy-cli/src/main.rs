//! TokenBuddy CLI (Rust): port do `tokenbuddy` Python (`__main__.py`).
//!
//! Subcomandos: serve, status, settings, navigate, event, expression, run, codex.
//!
//! `serve` abre a porta serial e roda o servidor HTTP local.
//! Os demais comandos falam com o host em execução via HTTP (mesmo padrão do Python).

use std::path::PathBuf;

use serde_json::{json, Value};
use tokenbuddy_core::bridge::Bridge;
use tokenbuddy_core::connectors;
use tokenbuddy_core::metrics::sample_metrics;

const VERSION: &str = "0.3.0";

/// Estados de expressão aceitos pelo firmware (mesma ordem de `behavior.h`).
pub const STATES: &[(&str, i32)] = &[
    ("idle", 0),
    ("happy", 1),
    ("thinking", 2),
    ("error", 3),
    ("curious", 4),
    ("working", 5),
    ("success", 6),
    ("sleeping", 7),
    ("disconnected", 8),
    ("dizzy", 9),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = match args.get(1).map(|s| s.as_str()) {
        Some("serve") => cmd_serve(&args[2..]),
        Some("status") => cmd_status(),
        Some("settings") => cmd_settings(&args[2..]),
        Some("navigate") => cmd_navigate(&args[2..]),
        Some("event") => cmd_event(&args[2..]),
        Some("expression") => cmd_expression(&args[2..]),
        Some("run") => cmd_run(&args[2..]),
        Some("codex") => cmd_codex(&args[2..]),
        _ => {
            print_usage();
            2
        }
    };
    std::process::exit(code);
}

fn print_usage() {
    eprintln!("TokenBuddy CLI v{VERSION}\n\nUsos:\n  serve [--port <porta>] [--mac <MAC>]\n  status\n  settings [--brightness N] [--animation N] [--demo true|false]\n  navigate --page N\n  event --event <agent.started|agent.completed|agent.failed> --agent <nome> [--input-tokens N] [--output-tokens N] [--task-id ID]\n  expression --state <estado> [--duration ms]\n  run -- <comando...>\n  codex <argumentos...>\n");
}

fn get_flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name && i + 1 < args.len() {
            return Some(&args[i + 1]);
        }
        i += 1;
    }
    None
}

fn load_config() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join(".tools")
        .join("host-config.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(json!({"port": "COM3", "mac": "", "http_port": 8765, "api_key": ""}))
}

// ---------------------------------------------------------------------------
// HTTP client — fala com o host em execução (mesmo padrão do Python)
// ---------------------------------------------------------------------------

fn http_request(config: &Value, route: &str, payload: Option<&Value>) -> Result<Value, String> {
    let port = config["http_port"].as_u64().unwrap_or(8765);
    let key = config["api_key"].as_str().unwrap_or("");
    let url = format!("http://127.0.0.1:{port}{route}");

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(6))
        .build();

    let req = match payload {
        Some(body) => agent
            .post(&url)
            .set("X-TokenBuddy-Key", key)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string()),
        None => agent.get(&url).set("X-TokenBuddy-Key", key).call(),
    };

    match req {
        Ok(response) => response
            .into_json::<Value>()
            .map_err(|e| format!("Resposta inválida: {e}")),
        Err(ureq::Error::Status(code, _)) if code == 401 => {
            Err("Chave local inválida ou ausente".into())
        }
        Err(e) => Err(format!("{e}")),
    }
}

// ---------------------------------------------------------------------------
// Subcomandos
// ---------------------------------------------------------------------------

fn cmd_serve(args: &[String]) -> i32 {
    let port = get_flag(args, "--port").map(|s| s.to_string());
    let mac = get_flag(args, "--mac").map(|s| s.to_string());
    let config = load_config();
    let port = port.unwrap_or_else(|| config["port"].as_str().unwrap_or("COM3").to_string());
    let mac = mac.unwrap_or_else(|| config["mac"].as_str().unwrap_or("").to_string());

    eprintln!("TokenBuddy serve: porta={port}");
    eprintln!("Aguardando dispositivo...");

    let mut bridge = Bridge::new(&port, &mac);
    bridge.start(move || sample_metrics());

    // Mantém vivo até Ctrl+C.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn cmd_status() -> i32 {
    let config = load_config();
    match http_request(&config, "/api/status", None) {
        Ok(v) => {
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            0
        }
        Err(e) => {
            eprintln!("Erro: {e}");
            4
        }
    }
}

fn cmd_settings(args: &[String]) -> i32 {
    let mut message = json!({"type": "settings"});
    if let Some(b) = get_flag(args, "--brightness").and_then(|s| s.parse::<u64>().ok()) {
        if b > 50 {
            eprintln!("brightness excede o limite (0..=50)");
            return 2;
        }
        message["brightness"] = json!(b);
    }
    if let Some(a) = get_flag(args, "--animation").and_then(|s| s.parse::<u64>().ok()) {
        if a > 2 {
            eprintln!("animation excede o limite (0..=2)");
            return 2;
        }
        message["animation"] = json!(a);
    }
    if let Some(d) = get_flag(args, "--demo").and_then(|s| s.parse::<bool>().ok()) {
        message["demo"] = json!(d);
    }
    send_command(&message)
}

fn cmd_navigate(args: &[String]) -> i32 {
    let page = match get_flag(args, "--page").and_then(|s| s.parse::<u64>().ok()) {
        Some(p) if p <= 4 => p,
        _ => {
            eprintln!("navigate requer --page 0..=4");
            return 2;
        }
    };
    send_command(&json!({"type": "navigate", "page": page}))
}

fn cmd_event(args: &[String]) -> i32 {
    let event = match get_flag(args, "--event") {
        Some(e) if ["agent.started", "agent.completed", "agent.failed"].contains(&e) => e.to_string(),
        _ => {
            eprintln!("event requer --event valido");
            return 2;
        }
    };
    let agent = match get_flag(args, "--agent") {
        Some(a) if !a.is_empty() && a.len() <= 31 => a.to_string(),
        _ => {
            eprintln!("event requer --agent (1..=31 chars)");
            return 2;
        }
    };
    let mut message = json!({"type": "event", "event": event, "agent": agent});
    if let Some(i) = get_flag(args, "--input-tokens").and_then(|s| s.parse::<u64>().ok()) {
        message["input_tokens"] = json!(i);
    }
    if let Some(o) = get_flag(args, "--output-tokens").and_then(|s| s.parse::<u64>().ok()) {
        message["output_tokens"] = json!(o);
    }
    if let Some(t) = get_flag(args, "--task-id") {
        message["task_id"] = json!(t);
    }
    send_command(&message)
}

fn cmd_expression(args: &[String]) -> i32 {
    let state_name = match get_flag(args, "--state") {
        Some(s) => s.to_string(),
        None => {
            eprintln!("expression requer --state");
            return 2;
        }
    };
    if !STATES.iter().any(|(name, _)| *name == state_name) {
        eprintln!("state invalido: {state_name}");
        return 2;
    }
    let duration = get_flag(args, "--duration").and_then(|s| s.parse::<u64>().ok()).unwrap_or(3000);
    if duration < 50 || duration > 60_000 {
        eprintln!("duration deve estar entre 50 e 60000");
        return 2;
    }
    send_command(&json!({"type": "expression", "state": state_name, "duration": duration}))
}

fn cmd_run(args: &[String]) -> i32 {
    let args: Vec<String> = if args.first().map(|s| s.as_str()) == Some("--") {
        args[1..].to_vec()
    } else {
        args.to_vec()
    };
    if args.is_empty() {
        eprintln!("run requer um comando");
        return 2;
    }
    let config = load_config();
    let publish = |event: Value| {
        let _ = http_request(&config, "/api/event", Some(&event));
    };
    connectors::run_command(&args, "CLI", &publish)
}

fn cmd_codex(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("codex requer argumentos");
        return 2;
    }
    let config = load_config();
    let publish = |event: Value| {
        let _ = http_request(&config, "/api/event", Some(&event));
    };
    connectors::run_codex(args, &publish)
}

fn send_command(message: &Value) -> i32 {
    let config = load_config();
    // A API HTTP usa o formato Pydantic (sem "type"), não o formato interno da bridge.
    let (route, payload): (&str, Value) = match message["type"].as_str() {
        Some("settings") => {
            let mut p = json!({});
            for key in ["brightness", "animation", "demo"] {
                if let Some(v) = message.get(key) {
                    p[key] = v.clone();
                }
            }
            ("/api/settings", p)
        }
        Some("navigate") => ("/api/navigate", json!({"page": message["page"]})),
        Some("event") => {
            let mut p = json!({});
            for key in ["event", "agent", "input_tokens", "output_tokens", "task_id"] {
                if let Some(v) = message.get(key) {
                    p[key] = v.clone();
                }
            }
            ("/api/event", p)
        }
        Some("expression") => (
            "/api/expression",
            json!({"state": message["state"], "duration": message["duration"]}),
        ),
        _ => return 2,
    };
    match http_request(&config, route, Some(&payload)) {
        Ok(v) => {
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            0
        }
        Err(e) => {
            eprintln!("Erro: {e}");
            4
        }
    }
}
