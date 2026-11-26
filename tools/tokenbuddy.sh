#!/bin/bash
# Wrapper do CLI TokenBuddy para Linux. Equivalente ao tokenbuddy.ps1.
# Uso: ./tools/tokenbuddy.sh <subcomando> [args...]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cli="$root/services/host/target/release/tokenbuddy-cli"

if [ ! -x "$cli" ]; then
    echo "Execute ./tools/host.sh setup primeiro." >&2
    exit 1
fi

"$cli" "$@"
