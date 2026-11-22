$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:IDF_TOOLS_PATH = Join-Path $projectRoot '.tools/espressif'
$env:IDF_PATH = Join-Path $projectRoot '.tools/esp-idf-v5.5.2'
$env:IDF_TARGET = 'esp32c6'
if (-not (Test-Path "$env:IDF_PATH/export.ps1")) {
    throw 'SDK local ausente. Consulte docs/setup-windows.md.'
}
$activationPreference = $ErrorActionPreference
try {
    # SDK writes activation progress to stderr, especially visible with *> logs.
    $ErrorActionPreference = 'Continue'
    . "$env:IDF_PATH/export.ps1"
    $activationExit = $LASTEXITCODE
} finally { $ErrorActionPreference = $activationPreference }
if ($activationExit -ne 0) { throw 'Falha ao ativar ESP-IDF.' }
