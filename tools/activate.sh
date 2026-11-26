#!/bin/bash
# Ativa o ESP-IDF v5.5.2 local em .tools/ para o processo corrente.
# Equivalente ao activate.ps1 (Windows). Não roda o build; apenas configura o ambiente.
# Uso: . ./tools/activate.sh

projectRoot="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export IDF_TOOLS_PATH="$projectRoot/.tools/espressif"
export IDF_PATH="$projectRoot/.tools/esp-idf-v5.5.2"
export IDF_TARGET=esp32c6

if [ ! -f "$IDF_PATH/export.sh" ]; then
    echo "SDK local ausente em $IDF_PATH." >&2
    echo "Consulte docs/setup-windows.md para o fluxo (clone da tag v5.5.2 + install.sh)." >&2
    return 1 2>/dev/null || exit 1
fi

# export.sh escreve progresso em stderr; o pipefail não deve quebrar a ativação.
export IDF_TOOLS_PATH IDF_PATH IDF_TARGET
. "$IDF_PATH/export.sh" >&2
idfExit=$?
if [ "$idfExit" -ne 0 ]; then
    echo "Falha ao ativar ESP-IDF (exit $idfExit)." >&2
    return 1 2>/dev/null || exit 1
fi

echo "ESP-IDF ativado: $IDF_PATH (target: $IDF_TARGET)"
