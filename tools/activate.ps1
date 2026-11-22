$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:IDF_TOOLS_PATH = Join-Path $projectRoot '.tools/espressif'
$env:IDF_PATH = Join-Path $projectRoot '.tools/esp-idf-v5.5.2'
$env:IDF_TARGET = 'esp32c6'
if (-not (Test-Path "$env:IDF_PATH/export.ps1")) {
    throw 'SDK local ausente. Consulte docs/setup-windows.md.'
}
. "$env:IDF_PATH/export.ps1"
if ($LASTEXITCODE -ne 0) { throw 'Falha ao ativar ESP-IDF.' }
