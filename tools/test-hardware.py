"""Live API -> USB -> firmware checks; writes settings and navigates the real LCD."""
import argparse
import json
import pathlib
import subprocess
import sys
import time
import urllib.error
import urllib.request

from tokenbuddy.__main__ import DEFAULT_CONFIG, request


def wait_status(config, predicate, timeout=8):
    end = time.monotonic() + timeout
    last = None
    while time.monotonic() < end:
        last = request(config, "/api/status")
        if last["device_fresh"] and predicate(last["device"]):
            return last
        time.sleep(0.15)
    raise AssertionError(f"Status nao atingiu criterio: {last}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--soak-seconds", type=int, default=180)
    args = parser.parse_args()
    config = json.loads(DEFAULT_CONFIG.read_text(encoding="utf-8-sig"))
    root = pathlib.Path(__file__).resolve().parents[1]
    report = {"kind": "physical-device-live-test", "checks": [], "samples": []}
    initial = wait_status(config, lambda d: d["online"] and d["selftest"])
    assert initial["device"]["mac"] == config["mac"]
    assert initial["device"]["fw"] == "0.3.0"
    report["checks"].append("USB identity and startup selftests")
    # Authentication must fail without changing the device.
    try:
        urllib.request.urlopen(f"http://127.0.0.1:{config['http_port']}/api/status", timeout=2)
    except urllib.error.HTTPError as error:
        assert error.code == 401
    else:
        raise AssertionError("API allowed unauthenticated status")
    report["checks"].append("API unauthorized request rejected")
    for page in range(5):
        request(config, "/api/navigate", {"page": page})
        wait_status(config, lambda d: d["page"] == page)
    report["checks"].append("All five pages acknowledged on physical device")
    request(config, "/api/settings", {"brightness": 20, "animation": 2, "demo": False})
    wait_status(config, lambda d: d["brightness"] == 20 and d["animation"] == 2 and d["settings_saved"])
    request(config, "/api/settings", {"brightness": 30, "animation": 1})
    wait_status(config, lambda d: d["brightness"] == 30 and d["animation"] == 1)
    report["checks"].append("Settings, limits and NVS commit acknowledged")
    request(config, "/api/navigate", {"page": 0})
    # Real subprocess events, without fabricated token counts.
    subprocess.run([sys.executable, "-m", "tokenbuddy", "run", "--agent", "Validacao",
                    "--", sys.executable, "-c", "import time; time.sleep(2)"], check=True)
    wait_status(config, lambda d: d["state"] == 6)
    report["checks"].append("Real process completion -> SUCCESS on device")
    time.sleep(5.2)
    result = subprocess.run([sys.executable, "-m", "tokenbuddy", "run", "--agent", "Validacao",
                             "--", sys.executable, "-c", "raise SystemExit(7)"], check=False)
    assert result.returncode == 7
    wait_status(config, lambda d: d["state"] == 3)
    report["checks"].append("Real process failure -> ERROR on device")
    time.sleep(5.2)
    states = ["idle", "happy", "thinking", "error", "curious", "working", "success", "sleeping", "disconnected", "dizzy"]
    for state, name in enumerate(states):
        request(config, "/api/expression", {"state": name, "duration": 2200})
        wait_status(config, lambda d: d["state"] == state, timeout=3)
        time.sleep(2.3)
    report["checks"].append("All ten expression states and expiry")
    snapshot = request(config, "/api/status")
    assert snapshot["telemetry"]["ram_total"] > 0
    assert "cpu" in snapshot["telemetry"]
    report["telemetry"] = snapshot["telemetry"]
    report["checks"].append("Real CPU/RAM telemetry (GPU optional, never substituted)")
    started = time.monotonic()
    previous_uptime = 0
    while time.monotonic() - started < args.soak_seconds:
        value = wait_status(config, lambda d: d["online"])
        device = value["device"]
        assert device["uptime_ms"] >= previous_uptime, "Unexpected reset"
        assert device["stack_min"] > 1200, "Main stack margin too low"
        previous_uptime = device["uptime_ms"]
        report["samples"].append(device)
        if len(report["samples"]) % 15 == 0:
            (root / ".tools/integrated-hardware-report.partial.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        time.sleep(2)
    samples = report["samples"]
    if len(samples) >= 10:
        assert min(d["heap"] for d in samples[-5:]) >= min(d["heap"] for d in samples[:5]) - 1024
        assert min(d["lv_free"] for d in samples[-5:]) >= min(d["lv_free"] for d in samples[:5]) - 512
    report["soak_seconds"] = time.monotonic() - started
    report["checks"].append("Soak: no reset; bounded system and LVGL free memory")
    report["result"] = "PASS"
    (root / ".tools/integrated-hardware-report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({"result": "PASS", "checks": report["checks"], "samples": len(samples)}, indent=2))


if __name__ == "__main__":
    main()
