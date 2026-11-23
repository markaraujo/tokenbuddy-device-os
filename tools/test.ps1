$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
& "$PSScriptRoot/test-behavior.ps1"
$cargo = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
if (-not (Test-Path $cargo)) { throw 'cargo nao encontrado. Instale o Rust: https://rustup.rs' }
$previous=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    & $cargo test --manifest-path "$root/services/host/Cargo.toml"
    $testExit=$LASTEXITCODE
} finally {$ErrorActionPreference=$previous}
if ($testExit -ne 0) {throw 'Testes Rust falharam.'}
