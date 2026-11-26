#!/bin/bash
# Flash deliberado do firmware — exige confirmação explícita. Equivalente ao flash.ps1.
# NENHUMA gravação acontece sem -Confirmed após conferir placa/porta.
# Uso: ./tools/flash.sh <Porta> -ExpectedMac <MAC> -Confirmed
# Exemplo: ./tools/flash.sh /dev/ttyACM0 -ExpectedMac 14:c1:9f:05:7b:ec -Confirmed
set -euo pipefail

port=""
expectedMac=""
confirmed=false
while [ $# -gt 0 ]; do
    case "$1" in
        -ExpectedMac) expectedMac="$2"; shift 2 ;;
        -ExpectedMac=*) expectedMac="${1#-ExpectedMac=}"; shift ;;
        -Confirmed) confirmed=true; shift ;;
        -h|--help) sed -n '2,5p' "$0"; exit 0 ;;
        *)
            if [ -z "$port" ]; then port="$1"; shift
            else echo "Argumento não reconhecido: $1" >&2; exit 2
            fi ;;
    esac
done

if [ -z "$port" ] || [ -z "$expectedMac" ]; then
    echo "Uso: $0 <Porta> -ExpectedMac <MAC> -Confirmed" >&2
    exit 2
fi
if [ -z "$expectedMac" ]; then
    echo "Expectacionar MAC exigida (ex.: 14:c1:9f:05:7b:ec)." >&2
    exit 2
fi
if [ "$confirmed" != true ]; then
    echo "Confirme explicitamente com -Confirmed após conferir placa e porta." >&2
    exit 2
fi

# Mesma validação de MAC/flash/chip do flash.ps1
if ! [[ "$expectedMac" =~ ^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$ ]]; then
    echo "MAC inválido: $expectedMac" >&2
    exit 2
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
. "$root/tools/activate.sh" >&2

sdkVersion="$(git -C "$IDF_PATH" describe --tags --dirty 2>/dev/null)"
if [ "$sdkVersion" != "v5.5.2" ]; then
    echo "SDK diverge da tag limpa v5.5.2 (encontrado: $sdkVersion)." >&2
    exit 1
fi

identity="$(python -m esptool --chip esp32c6 --port "$port" read_mac 2>&1)"
if ! printf '%s\n' "$identity" | grep -i -E "BASE MAC: $(echo "$expectedMac" | tr 'A-Z' 'a-z')" >/dev/null; then
    echo "Chip/MAC não confirmado; gravação cancelada." >&2
    printf '%s\n' "$identity" >&2
    exit 1
fi

flashIdentity="$(python -m esptool --chip esp32c6 --port "$port" flash_id 2>&1)"
if ! printf '%s\n' "$flashIdentity" | grep -q "Detected flash size: 8MB"; then
    echo "Esta versão exige a unidade de 8 MB validada." >&2
    printf '%s\n' "$flashIdentity" >&2
    exit 1
fi

python "$IDF_PATH/tools/idf.py" -C "$root/firmware/esp32-c6" -p "$port" flash
