import argparse
import json
import pathlib
import sys
import urllib.error
import urllib.request

from .api import create_app
from .bridge import Bridge
from .connectors import run_codex, run_command

DEFAULT_CONFIG = pathlib.Path(__file__).resolve().parents[3] / ".tools" / "host-config.json"


def request(config, route, payload=None):
    body = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(f"http://127.0.0.1:{config['http_port']}{route}", data=body,
        headers={"X-TokenBuddy-Key": config["api_key"], "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=6) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser(description="TokenBuddy: host USB e eventos locais")
    parser.add_argument("--config", type=pathlib.Path, default=DEFAULT_CONFIG)
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("serve")
    sub.add_parser("status")
    settings = sub.add_parser("settings")
    settings.add_argument("--brightness", type=int, choices=range(0, 51))
    settings.add_argument("--animation", type=int, choices=range(3))
    settings.add_argument("--demo", choices=["on", "off"])
    nav = sub.add_parser("navigate")
    nav.add_argument("page", type=int, choices=range(5))
    event = sub.add_parser("event")
    event.add_argument("event", choices=["agent.started", "agent.completed", "agent.failed"])
    event.add_argument("--agent", default="Manual")
    event.add_argument("--input-tokens", type=int)
    event.add_argument("--output-tokens", type=int)
    expression = sub.add_parser("expression")
    from .api import STATES
    expression.add_argument("state", choices=STATES)
    expression.add_argument("--duration", type=int, default=3000)
    run = sub.add_parser("run")
    run.add_argument("--agent", default="Processo")
    run.add_argument("command", nargs=argparse.REMAINDER)
    codex = sub.add_parser("codex")
    codex.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        config = json.loads(args.config.read_text(encoding="utf-8-sig"))
        if args.action == "serve":
            import uvicorn
            if len(config["api_key"]) < 24:
                raise ValueError("Chave local deve ter pelo menos 24 caracteres")
            bridge = Bridge(config["port"], config["mac"])
            uvicorn.run(create_app(bridge, config["api_key"]), host="127.0.0.1", port=config["http_port"])
            return
        if args.action == "status":
            result = request(config, "/api/status")
        elif args.action == "settings":
            payload = {key: getattr(args, key) for key in ("brightness", "animation") if getattr(args, key) is not None}
            if args.demo is not None:
                payload["demo"] = args.demo == "on"
            result = request(config, "/api/settings", payload)
        elif args.action == "navigate":
            result = request(config, "/api/navigate", {"page": args.page})
        elif args.action == "event":
            payload = {"event": args.event, "agent": args.agent}
            for key in ("input_tokens", "output_tokens"):
                if getattr(args, key) is not None:
                    payload[key] = getattr(args, key)
            result = request(config, "/api/event", payload)
        elif args.action == "expression":
            result = request(config, "/api/expression", {"state": args.state, "duration": args.duration})
        else:
            def publish(event):
                try:
                    request(config, "/api/event", event)
                except (OSError, ValueError) as error:
                    print(f"TokenBuddy indisponivel: {error}", file=sys.stderr)
            arguments = args.arguments if args.action == "codex" else args.command
            if arguments and arguments[0] == "--":
                arguments = arguments[1:]
            if not arguments:
                parser.error("Forneca os argumentos depois de --")
            raise SystemExit(run_codex(arguments, publish) if args.action == "codex" else run_command(arguments, args.agent, publish))
        print(json.dumps(result, indent=2, ensure_ascii=False))
    except (OSError, ValueError, KeyError) as error:
        if isinstance(error, urllib.error.HTTPError):
            print(error.read().decode(errors="replace"), file=sys.stderr)
        else:
            print(str(error), file=sys.stderr)
        raise SystemExit(1) from error


if __name__ == "__main__":
    main()
