"""Official Codex exec JSONL events. Never infer usage from terminal text."""
import json
import subprocess
import sys
import uuid


def codex_event(value):
    kind = value.get("type")
    if kind == "turn.started":
        return {"event": "agent.started", "agent": "Codex"}
    if kind == "turn.failed":
        return {"event": "agent.failed", "agent": "Codex"}
    if kind == "turn.completed":
        result = {"event": "agent.completed", "agent": "Codex"}
        usage = value.get("usage") or {}
        counts = [usage.get("input_tokens"), usage.get("output_tokens")]
        if all(type(v) is int and 0 <= v <= 2147483647 for v in counts):
            result.update(input_tokens=counts[0], output_tokens=counts[1])
        return result
    return None


def run_codex(arguments, publish):
    task = uuid.uuid4().hex
    process = subprocess.Popen(["codex", "exec", "--json", *arguments], stdout=subprocess.PIPE,
                               text=True, encoding="utf-8", errors="replace")
    published_terminal = False
    for line in process.stdout:
        print(line, end="", flush=True)
        try:
            value = json.loads(line)
            event = codex_event(value) if isinstance(value, dict) else None
        except ValueError:
            event = None
        if event:
            publish({**event, "task_id": task})
            published_terminal = event["event"] != "agent.started"
    result = process.wait()
    if result and not published_terminal:
        publish({"event": "agent.failed", "agent": "Codex", "task_id": task})
    return result


def run_command(arguments, agent, publish):
    task = uuid.uuid4().hex
    publish({"event": "agent.started", "agent": agent, "task_id": task})
    try:
        result = subprocess.run(arguments, check=False).returncode
    except (OSError, KeyboardInterrupt):
        publish({"event": "agent.failed", "agent": agent, "task_id": task})
        raise
    publish({"event": "agent.completed" if result == 0 else "agent.failed", "agent": agent, "task_id": task})
    return result
