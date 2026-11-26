#!/bin/bash
# Wrap idf.py para o firmware ESP32-C6. Equivalente ao idf.ps1 (Windows).
# Uso: . ./tools/activate.sh && ./tools/idf.sh [build|menuconfig|size]
set -euo pipefail

action="${1:-build}"
case "$action" in
    build|menuconfig|size) ;;
    *) echo "Uso: $0 [build|menuconfig|size]" >&2; exit 2 ;;
esac

if [ -z "${IDF_PATH:-}" ] || [ ! -f "$IDF_PATH/tools/idf.py" ]; then
    echo "ESP-IDF indisponível. Ative o ambiente primeiro: . ./tools/activate.sh" >&2
    exit 1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root/firmware/esp32-c6"
python "$IDF_PATH/tools/idf.py" "$action"
