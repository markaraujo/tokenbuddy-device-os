param([Parameter(ValueFromRemainingArguments=$true)][string[]]$CliArguments)
$root=Split-Path $PSScriptRoot -Parent
$cliExe=Join-Path $root 'services/host/target/release/tokenbuddy-cli.exe'
if (-not (Test-Path $cliExe)) {throw 'Execute ./tools/host.ps1 setup primeiro.'}
$ErrorActionPreference='Continue'
& $cliExe @CliArguments
exit $LASTEXITCODE
