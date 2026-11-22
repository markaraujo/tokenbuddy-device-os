param([ValidateSet('build','menuconfig','size')][string]$Action = 'build')
$ErrorActionPreference = 'Stop'
if (-not $env:IDF_PATH -or -not (Test-Path "$env:IDF_PATH/tools/idf.py")) {
    throw 'ESP-IDF indisponível. Execute export.ps1 da instalação v5.5.2 antes deste comando.'
}
$pythonCommand = Get-Command python -ErrorAction Stop
Push-Location (Join-Path $PSScriptRoot '../firmware/esp32-c6')
try {
    # Windows PowerShell transforma stderr nativo em ErrorRecord mesmo com exit 0.
    $previousErrorAction = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & $pythonCommand.Source "$env:IDF_PATH/tools/idf.py" $Action
        $idfExitCode = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previousErrorAction }
    if ($idfExitCode -ne 0) { throw "idf.py falhou: $idfExitCode" }
} finally { Pop-Location }
