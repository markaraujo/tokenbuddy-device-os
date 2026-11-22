import copy
import json
import queue
import threading
import time
from dataclasses import dataclass, field

import serial
from serial.tools import list_ports

from .metrics import sample_metrics


class CommandError(RuntimeError):
    pass


@dataclass
class Pending:
    message: dict
    deadline: float
    finished: threading.Event = field(default_factory=threading.Event)
    result: dict | None = None
    error: Exception | None = None


class Bridge:
    def __init__(self, port: str, expected_mac: str):
        self.port, self.expected_mac = port, expected_mac.lower()
        self.lock = threading.Lock()
        self.commands = queue.Queue(maxsize=32)
        self.stopping = threading.Event()
        self.thread = None
        self.serial = None
        self.connected = False
        self.device = {}
        self.telemetry = {}
        self.last_received = 0.0
        self.last_error = None
        self.errors = []
        self.last_agent = None
        self.next_id = 1
        self._rx = bytearray()

    def start(self):
        self.stopping.clear()
        self.thread = threading.Thread(target=self._worker, name="tokenbuddy-usb", daemon=True)
        self.thread.start()

    def stop(self):
        self.stopping.set()
        if self.thread:
            self.thread.join(timeout=6)

    def snapshot(self):
        with self.lock:
            return copy.deepcopy({"serial_connected": self.connected,
                "device_fresh": self.connected and time.monotonic() - self.last_received < 3,
                "device": self.device, "telemetry": self.telemetry,
                "last_error": self.last_error, "last_agent_event": self.last_agent,
                "recent_errors": self.errors,
                "queued": self.commands.qsize()})

    def request(self, message: dict) -> dict:
        with self.lock:
            if not self.connected:
                raise ConnectionError("Dispositivo USB desconectado ou identidade nao confirmada")
        pending = Pending(copy.deepcopy(message), time.monotonic() + 3)
        try:
            self.commands.put_nowait(pending)
        except queue.Full as error:
            raise ConnectionError("Fila de comandos cheia") from error
        if not pending.finished.wait(4):
            raise TimeoutError("Dispositivo nao confirmou o comando")
        if pending.error:
            raise pending.error
        return pending.result

    def _read(self):
        chunk = self.serial.read(256)
        self._rx.extend(chunk)
        if len(self._rx) > 8192:
            self._rx.clear()
        messages = []
        while b"\n" in self._rx:
            line, _, rest = self._rx.partition(b"\n")
            self._rx = bytearray(rest)
            try:
                value = json.loads(line)
            except (ValueError, UnicodeError):
                continue
            if not isinstance(value, dict) or value.get("v") != 1:
                continue
            if value.get("type") == "status" and value.get("device") == "TokenBuddy":
                mac = value.get("mac")
                if not isinstance(mac, str):
                    continue
                if mac.lower() != self.expected_mac:
                    raise ConnectionError("MAC do dispositivo diferente da configurada")
                with self.lock:
                    self.device = value
                    self.last_received = time.monotonic()
            messages.append(value)
        return messages

    def _exchange(self, message):
        identifier = self.next_id
        self.next_id = 1 if identifier >= 2147483647 else identifier + 1
        wire = json.dumps({**message, "v": 1, "id": identifier}, separators=(",", ":"), allow_nan=False).encode("ascii")
        if len(wire) > 768:
            raise CommandError("Comando excede 768 bytes")
        self.serial.write(wire + b"\n")
        deadline = time.monotonic() + 1.5
        while time.monotonic() < deadline and not self.stopping.is_set():
            for value in self._read():
                if value.get("type") == "ack" and value.get("id") == identifier:
                    if value.get("ok") is not True:
                        raise CommandError(value.get("error", "device_rejected"))
                    return value
        raise TimeoutError("Sem ACK USB; nenhum retry automatico de evento")

    def _connect(self):
        ports = {port.device: port for port in list_ports.comports()}
        candidate = ports.get(self.port)
        if candidate is None or (candidate.vid, candidate.pid) != (0x303A, 0x1001):
            raise ConnectionError("Porta configurada nao e Espressif USB Serial/JTAG")
        self.serial = serial.Serial()
        self.serial.port = self.port
        self.serial.baudrate = 115200
        self.serial.timeout = 0.1
        self.serial.write_timeout = 1
        self.serial.dtr = self.serial.rts = False
        self.serial.open()
        self._rx.clear()
        started = time.monotonic()
        while time.monotonic() - started < 6 and not self.stopping.is_set():
            self._read()
            with self.lock:
                identified = self.last_received >= started and self.device.get("fw") == "0.3.0"
            if identified:
                self.serial.write(b"\n")  # End any partial frame left by a previous host.
                self._exchange({"type": "heartbeat"})
                with self.lock:
                    self.connected = True
                    self.last_error = None
                    agent = copy.deepcopy(self.last_agent)
                if agent and agent.get("event") == "agent.started":
                    self._exchange(agent)
                return
        raise ConnectionError("Firmware 0.3.0 / MAC esperado nao confirmado")

    def _worker(self):
        heartbeat = metrics_at = 0.0
        while not self.stopping.is_set():
            try:
                if self.serial is None:
                    self._connect()
                    heartbeat = metrics_at = 0.0
                now = time.monotonic()
                if now - heartbeat >= 1:
                    self._exchange({"type": "heartbeat"})
                    heartbeat = now
                if now - metrics_at >= 2:
                    metrics = sample_metrics()
                    self._exchange(metrics)
                    with self.lock:
                        self.telemetry = metrics
                    metrics_at = time.monotonic()
                for _ in range(4):
                    try:
                        pending = self.commands.get_nowait()
                    except queue.Empty:
                        break
                    try:
                        if time.monotonic() > pending.deadline:
                            raise TimeoutError("Comando expirou antes do envio")
                        if pending.message.get("type") == "event":
                            with self.lock:
                                active = copy.deepcopy(self.last_agent)
                            if active and active.get("event") == "agent.started":
                                incoming = pending.message
                                if incoming.get("agent") != active.get("agent") or incoming.get("task_id") != active.get("task_id"):
                                    raise CommandError("Outra tarefa esta ativa; MVP acompanha uma tarefa por vez")
                        pending.result = self._exchange(pending.message)
                        if pending.message.get("type") == "event":
                            with self.lock:
                                self.last_agent = pending.message
                    except Exception as error:
                        pending.error = error
                        if not isinstance(error, CommandError):
                            raise
                    finally:
                        pending.finished.set()
                self._read()
            except (OSError, ConnectionError, TimeoutError, serial.SerialException, ValueError, CommandError) as error:
                with self.lock:
                    self.connected = False
                    self.last_error = str(error)[:200]
                    self.errors.append({"at": time.time(), "error": self.last_error})
                    self.errors = self.errors[-16:]
                if self.serial:
                    self.serial.close()
                self.serial = None
                self.stopping.wait(1)
        if self.serial:
            self.serial.close()
        with self.lock:
            self.connected = False
