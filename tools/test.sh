#!/bin/bash
# Testes no host (behavior C + Rust) para Linux. Equivalente ao test.ps1.
# O TinyCC usado no Windows é substituído pelo gcc do sistema (mesmos flags).
# Uso: ./tools/test.sh
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

compiler="${CC:-gcc}"
binary="$root/.tools/test-behavior"

"$compiler" -DTOKENBUDDY_HOST_TEST \
    "-I$root/firmware/esp32-c6/components/behavior/include" \
    "-I$root/firmware/esp32-c6/components/control/include" \
    "-I$root/.tools/esp-idf-v5.5.2/components/json/cJSON" \
    "$root/firmware/esp32-c6/components/behavior/behavior.c" \
    "$root/firmware/esp32-c6/components/control/control.c" \
    "$root/.tools/esp-idf-v5.5.2/components/json/cJSON/cJSON.c" \
    "$root/firmware/esp32-c6/tests/test_behavior.c" \
    "$root/firmware/esp32-c6/tests/test_control.c" \
    -o "$binary"
echo "Compilação dos testes host: OK"

"$binary"
echo "Behavior host tests PASS"

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
    exit 1
fi

"$cargoBin" test --manifest-path "$root/services/host/Cargo.toml"
