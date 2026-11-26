#!/bin/bash
# Contrato de host (start/stop/setup/status) para Linux. Equivalente ao host.ps1.
# Uso: ./tools/host.sh [setup|start|stop|status]
set -euo pipefail

action="${1:-status}"
case "$action" in
    setup|start|stop|status) ;;
    *) echo "Uso: $0 [setup|start|stop|status]" >&2; exit 2 ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
server="$root/services/host/target/release/tokenbuddy-server"
cli="$root/services/host/target/release/tokenbuddy-cli"
pidPath="$root/.tools/host.pid"
configPath="$root/.tools/host-config.json"
logPath="$root/.tools/host.log"
parentDir="$root"

mkConfig() {
    # Mesma estrutura gerada pelo host.ps1 (setup): port/mac/http_port/api_key
    local port="${TOKENBUDDY_PORT:-}"
    if [ -z "$port" ]; then
        if [ -e /dev/ttyACM0 ]; then
            port=/dev/ttyACM0
        elif ls /dev/ttyACM* >/dev/null 2>&1; then
            port="$(ls /dev/ttyACM* | head -1)"
        else
            port=/dev/ttyACM0
            echo "Aviso: nenhuma porta serial detectada; padrão usado: $port" >&2
        fi
    fi
    python3 - "$configPath" "$port" <<'PY'
import json, secrets, sys
path, port = sys.argv[1], sys.argv[2]
cfg = {
    "port": port,
    "mac": "14:c1:9f:05:7b:ec",
    "http_port": 8765,
    "api_key": secrets.token_urlsafe(32),
}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
print(f"Config criada: {path} (port: {port}, mac: {cfg['mac']}, api_key: gerada)")
PY
}

mk() {
    cargoBin="${CARGO:-}"
    if [ -z "$cargoBin" ]; then
        if command -v cargo >/dev/null 2>&1; then
            cargoBin="$(command -v cargo)"
        elif [ -x "$HOME/.cargo/bin/cargo" ]; then
            cargoBin="$HOME/.cargo/bin/cargo"
        fi
    fi
    if [ -z "$cargoBin" ]; then
        echo "cargo não encontrado. Instale o Rust/cargo: https://rustup.rs" >&2
        return 127
    fi
    "$cargoBin" build --release --manifest-path "$parentDir/services/host/Cargo.toml"
}

if [ "$action" = setup ]; then
    mk
    if [ ! -f "$configPath" ]; then
        mkdir -p "$parentDir/.tools"
        mkConfig
    else
        echo "Config existente mantida: $configPath"
    fi
    exit 0
fi

if [ ! -x "$server" ]; then
    echo "Binário não encontrado: $server" >&2
    echo "Execute ./tools/host.sh setup primeiro." >&2
    exit 1
fi
if [ ! -f "$configPath" ]; then
    echo "Config ausente: $configPath" >&2
    echo "Execute ./tools/host.sh setup primeiro." >&2
    exit 1
fi

if [ "$action" = status ]; then
    "$cli" status
    exit $?
fi

readSavedPid() {
    if [ -f "$pidPath" ]; then
        local saved
        saved="$(cat "$pidPath")"
        if [[ "$saved" =~ ^[0-9]+$ ]] && [ "$saved" -gt 0 ]; then
            echo "$saved"
            return 0
        fi
    fi
    return 1
}

savedPid=""
savedPid="$(readSavedPid || true)"
hostRunning=false
if [ -n "$savedPid" ] && kill -0 "$savedPid" 2>/dev/null; then
    hostRunning=true
fi

if [ "$action" = stop ]; then
    if [ "$hostRunning" = true ]; then
        kill "$savedPid" 2>/dev/null || true
        echo "Host encerrado."
    else
        echo "Host não está ativo."
    fi
    echo "0" > "$pidPath"
    exit 0
fi

if [ "$hostRunning" = true ]; then
    echo "Host já está ativo (PID $savedPid)."
    exit 0
fi

if [ -n "${UNSLOTH_API_KEY:-}" ]; then
    export UNSLOTH_API_KEY
fi
nohup "$server" >"$logPath" 2>&1 &
newPid=$!
echo "$newPid" > "$pidPath"
echo "Host iniciado: PID $newPid, API 127.0.0.1:8765 (log: $logPath)"
