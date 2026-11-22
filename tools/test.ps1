$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
& "$PSScriptRoot/test-behavior.ps1"
$python=Join-Path $root '.tools/host-env/Scripts/python.exe'
if (-not (Test-Path $python)) {throw 'Ambiente host ausente; execute tools/host.ps1 setup.'}
$previous=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    & $python -m unittest discover -s "$root/services/core/tests" -v
    $testExit=$LASTEXITCODE
} finally {$ErrorActionPreference=$previous}
if ($testExit -ne 0) {throw 'Testes Python falharam.'}
