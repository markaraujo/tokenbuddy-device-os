param([Parameter(ValueFromRemainingArguments=$true)][string[]]$CliArguments)
$root=Split-Path $PSScriptRoot -Parent
$python=Join-Path $root '.tools/host-env/Scripts/python.exe'
if (-not (Test-Path $python)) {throw 'Execute ./tools/host.ps1 setup primeiro.'}
$ErrorActionPreference='Continue'
& $python -m tokenbuddy @CliArguments
exit $LASTEXITCODE
