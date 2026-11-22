import json
import subprocess
import sys
import unittest
from unittest.mock import patch

from fastapi.testclient import TestClient
from tokenbuddy.api import create_app
from tokenbuddy.bridge import Bridge, CommandError
from tokenbuddy.connectors import codex_event, run_command
from tokenbuddy.metrics import sample_metrics


class FakeBridge:
    def __init__(self):
        self.commands = []
        self.failure = None

    def snapshot(self):
        return {"test_double": True}

    def request(self, message):
        if self.failure:
            raise self.failure
        self.commands.append(message)
        return {"ok": True}


class APITests(unittest.TestCase):
    def setUp(self):
        self.bridge = FakeBridge()
        self.client = TestClient(create_app(self.bridge, "test-key", manage_bridge=False))
        self.headers = {"X-TokenBuddy-Key": "test-key"}

    def test_auth_and_limits(self):
        self.assertEqual(self.client.get("/health").status_code, 200)
        self.assertEqual(self.client.get("/api/status").status_code, 401)
        for payload in ({"brightness": 51}, {"brightness": -1}, {"brightness": True}, {"animation": 3}, {"oops": 1}):
            self.assertEqual(self.client.post("/api/settings", json=payload, headers=self.headers).status_code, 422)
        self.assertEqual(self.bridge.commands, [])

    def test_event_metrics_not_invented(self):
        payload = {"agent": "Codex", "event": "agent.completed"}
        self.assertEqual(self.client.post("/api/event", json=payload, headers=self.headers).status_code, 200)
        self.assertNotIn("input_tokens", self.bridge.commands[-1])
        payload["input_tokens"] = 50
        self.assertEqual(self.client.post("/api/event", json=payload, headers=self.headers).status_code, 422)
        payload["output_tokens"] = 4
        self.assertEqual(self.client.post("/api/event", json=payload, headers=self.headers).status_code, 200)

    def test_command_routes_and_offline(self):
        self.assertEqual(self.client.post("/api/navigate", json={"page": 4}, headers=self.headers).status_code, 200)
        self.assertEqual(self.bridge.commands[-1], {"type": "navigate", "page": 4})
        self.assertEqual(self.client.post("/api/expression", json={"state": "error"}, headers=self.headers).status_code, 200)
        self.assertEqual(self.bridge.commands[-1]["state"], 3)
        self.bridge.failure = ConnectionError("offline")
        self.assertEqual(self.client.post("/api/settings", json={"demo": False}, headers=self.headers).status_code, 503)
        self.bridge.failure = CommandError("priority")
        self.assertEqual(self.client.post("/api/expression", json={"state": "happy"}, headers=self.headers).status_code, 409)


class ConnectorTests(unittest.TestCase):
    def test_codex_official_events(self):
        self.assertEqual(codex_event({"type": "turn.started"})["event"], "agent.started")
        self.assertEqual(codex_event({"type": "turn.failed"})["event"], "agent.failed")
        self.assertIsNone(codex_event({"type": "item.completed"}))
        self.assertIsNone(codex_event({"type": "error"}))  # A diagnostic alone need not end the turn.
        event = codex_event({"type": "turn.completed", "usage": {"input_tokens": 10, "cached_input_tokens": 8, "output_tokens": 2}})
        self.assertEqual(event["input_tokens"], 10)
        self.assertEqual(event["output_tokens"], 2)
        self.assertNotIn("context", event)
        self.assertNotIn("input_tokens", codex_event({"type": "turn.completed"}))
        self.assertNotIn("input_tokens", codex_event({"type": "turn.completed", "usage": {"input_tokens": True, "output_tokens": 2}}))

    def test_real_process_success_and_failure(self):
        for code in (0, 7):
            events = []
            result = run_command([sys.executable, "-c", f"raise SystemExit({code})"], "Test", events.append)
            self.assertEqual(result, code)
            self.assertEqual(events[0]["event"], "agent.started")
            self.assertEqual(events[1]["event"], "agent.completed" if code == 0 else "agent.failed")
            self.assertEqual(events[0]["task_id"], events[1]["task_id"])

    def test_gpu_unavailable_is_absent(self):
        with patch("tokenbuddy.metrics.shutil.which", return_value=None):
            metrics = sample_metrics()
        self.assertNotIn("gpu", metrics)
        self.assertGreater(metrics["ram_total"], 0)


class FakeSerial:
    def __init__(self, chunks):
        self.chunks = iter(chunks)

    def read(self, _count):
        return next(self.chunks, b"")


class FramingTests(unittest.TestCase):
    def test_partial_lines_logs_and_identity(self):
        bridge = Bridge("TEST", "aa:bb:cc:dd:ee:ff")
        message = {"v": 1, "type": "status", "device": "TokenBuddy", "mac": "aa:bb:cc:dd:ee:ff"}
        wire = json.dumps(message).encode() + b"\n"
        bridge.serial = FakeSerial([b"log\nnot-json\n" + wire[:10], wire[10:]])
        self.assertEqual(bridge._read(), [])
        self.assertEqual(bridge._read(), [message])
        self.assertEqual(bridge.snapshot()["device"]["mac"], message["mac"])
        message["mac"] = "11:22:33:44:55:66"
        bridge.serial = FakeSerial([json.dumps(message).encode() + b"\n"])
        with self.assertRaises(ConnectionError):
            bridge._read()

    def test_queue_requires_identification(self):
        with self.assertRaises(ConnectionError):
            Bridge("TEST", "expected").request({"type": "heartbeat"})


if __name__ == "__main__":
    unittest.main()
