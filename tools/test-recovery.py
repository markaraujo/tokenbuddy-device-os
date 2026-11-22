"""Live serial malformed-frame/offline/reset checks. Host must be stopped first."""
import json
import pathlib
import time
import subprocess

import serial
from tokenbuddy.__main__ import DEFAULT_CONFIG


class Device:
    def __init__(self, port):
        self.serial = serial.Serial()
        self.serial.port = port
        self.serial.baudrate = 115200
        self.serial.timeout = 0.15
        self.serial.dtr = self.serial.rts = False
        self.serial.open()
        self.buffer = bytearray()

    def read_until(self, predicate, timeout=10):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            self.buffer.extend(self.serial.read(512))
            while b"\n" in self.buffer:
                line, _, rest = self.buffer.partition(b"\n")
                self.buffer = bytearray(rest)
                try:
                    message = json.loads(line)
                except (ValueError, UnicodeError):
                    continue
                if predicate(message):
                    return message
        raise AssertionError("Resposta esperada nao recebida")

    def command(self, identifier, **payload):
        wire = json.dumps(dict(v=1, id=identifier, **payload)).encode() + b"\n"
        self.serial.write(wire)
        return self.read_until(lambda m: m.get("type") == "ack" and m.get("id") == identifier)


def main():
    root = pathlib.Path(__file__).resolve().parents[1]
    config = json.loads(DEFAULT_CONFIG.read_text(encoding="utf-8-sig"))
    device = Device(config["port"])
    report = {"kind": "physical-device-recovery", "checks": []}
    try:
        state = device.read_until(lambda m: m.get("type") == "status" and not m["online"])
        assert state["mac"] == config["mac"] and state["state"] == 8
        report["checks"].append("Host stopped -> DISCONNECTED within TTL")
        device.serial.write(b"x" * 1000 + b"\n")
        assert not device.read_until(lambda m: m.get("type") == "ack")["ok"]
        assert device.command(3001, type="heartbeat")["ok"]
        report["checks"].append("Oversized frame rejected; next valid frame accepted")
        device.serial.write(b"{broken\n")
        assert not device.read_until(lambda m: m.get("type") == "ack")["ok"]
        assert not device.command(3002, type="settings", brightness=99)["ok"]
        assert not device.command(3003, type="navigate", page=99)["ok"]
        report["checks"].append("Malformed JSON and out-of-range settings/navigation rejected")
        assert device.command(3004, type="settings", brightness=20, animation=2)["ok"]
        before = device.read_until(lambda m: m.get("type") == "status" and m["brightness"] == 20 and m["settings_saved"])
        assert before["animation"] == 2
    finally:
        device.serial.close()
    sdk_python = next((root / ".tools/espressif/python_env").glob("idf5.5_*/Scripts/python.exe"))
    result = subprocess.run([str(sdk_python), "-m", "esptool", "--chip", "esp32c6", "--port", config["port"], "--after", "hard_reset", "run"], capture_output=True, check=True)
    (root / ".tools/recovery-reset.log").write_bytes(result.stdout + result.stderr)
    time.sleep(1)
    end = time.monotonic() + 10
    while True:
        try:
            device = Device(config["port"])
            break
        except serial.SerialException:
            if time.monotonic() > end:
                raise
            time.sleep(0.5)
    try:
        after = device.read_until(lambda m: m.get("type") == "status")
        assert after["mac"] == config["mac"] and after["brightness"] == 20 and after["animation"] == 2
        assert after["uptime_ms"] < before["uptime_ms"] and after["selftest"]
        report["checks"].append("Reset rebooted firmware; brightness/animation survived NVS")
        assert device.command(3005, type="settings", brightness=30, animation=1, demo=False)["ok"]
        device.read_until(lambda m: m.get("type") == "status" and m["brightness"] == 30 and m["animation"] == 1)
        report["checks"].append("Restored normal settings after reset")
    finally:
        device.serial.close()
    report["result"] = "PASS"
    (root / ".tools/recovery-hardware-report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
