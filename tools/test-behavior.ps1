$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$compiler = Join-Path $root '.tools/tinycc/tcc/tcc.exe'
if (-not (Test-Path $compiler)) { throw 'TinyCC local ausente. Consulte docs/avatar-mvp.md.' }
$binary = Join-Path $root '.tools/test-behavior.exe'
& $compiler -DTOKENBUDDY_HOST_TEST "-I$root/firmware/esp32-c6/components/behavior/include" "$root/firmware/esp32-c6/components/behavior/behavior.c" "$root/firmware/esp32-c6/tests/test_behavior.c" -o $binary
if ($LASTEXITCODE -ne 0) { throw 'Falha ao compilar testes host.' }
& $binary
if ($LASTEXITCODE -ne 0) { throw 'Testes de comportamento falharam.' }
Write-Output 'Behavior host tests PASS'
