param([ValidateSet('setup','start','stop','status')][string]$Action='status')
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$serverExe=Join-Path $root 'services/host/target/release/tokenbuddy-server.exe'
$cliExe=Join-Path $root 'services/host/target/release/tokenbuddy-cli.exe'
$pidPath=Join-Path $root '.tools/host.pid'
$configPath=Join-Path $root '.tools/host-config.json'

function Test-HostProcess([long]$ProcId) {
    try {
        $p = Get-CimInstance Win32_Process -Filter "ProcessId=$ProcId" -ErrorAction Stop
        return ($p.ExecutablePath -eq $serverExe)
    } catch { return $false }
}

if ($Action -eq 'setup') {
    # Build release do workspace Rust
    $cargo = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
    if (-not (Test-Path $cargo)) { throw 'cargo nao encontrado. Instale o Rust: https://rustup.rs' }
    & $cargo build --release --manifest-path "$root/services/host/Cargo.toml"
    if ($LASTEXITCODE -ne 0) { throw 'Build do host Rust falhou.' }
    # Config (mesma estrutura que o Python usava)
    if (-not (Test-Path $configPath)) {
        $randomBytes=New-Object byte[] 32
        $generator=[Security.Cryptography.RandomNumberGenerator]::Create()
        try { $generator.GetBytes($randomBytes) } finally { $generator.Dispose() }
        @{port='COM3';mac='14:c1:9f:05:7b:ec';http_port=8765;api_key=[Convert]::ToBase64String($randomBytes)} | ConvertTo-Json | Set-Content -Encoding UTF8 $configPath
    }
    exit 0
}

if (-not (Test-Path $serverExe)) { throw 'Binario nao encontrado. Execute ./tools/host.ps1 setup primeiro.' }
if (-not (Test-Path $configPath)) { throw 'Config ausente. Execute ./tools/host.ps1 setup primeiro.' }

# Status: usa o CLI Rust via HTTP
if ($Action -eq 'status') { & $cliExe status; exit $LASTEXITCODE }

$existing=$null
if (Test-Path $pidPath) {
    $savedPid=Get-Content $pidPath -Raw
    if ($savedPid.Trim() -match '^\d+$' -and [long]$savedPid.Trim() -gt 0) {
        $procId=[long]$savedPid.Trim()
        if (Test-HostProcess $procId) { $existing=Get-CimInstance Win32_Process -Filter "ProcessId=$procId" }
    }
}

if ($Action -eq 'stop') {
    if ($existing) {
        Stop-Process -Id $existing.ProcessId -ErrorAction SilentlyContinue
        'Host encerrado.'
    } else {
        'Host nao esta ativo.'
    }
    Set-Content $pidPath '0'
    exit 0
}

if ($existing) { Write-Output 'Host ja esta ativo.'; exit 0 }

# IA Local (Unsloth/llama.cpp): usa a mesma chave do OpenCode, se definida.
$envVars=@{}
if ($env:UNSLOTH_API_KEY) { $envVars['UNSLOTH_API_KEY']=$env:UNSLOTH_API_KEY }

$psi=New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName=$serverExe
$psi.WorkingDirectory=$root
$psi.UseShellExecute=$false
$psi.RedirectStandardOutput=$true
$psi.RedirectStandardError=$true
foreach ($k in $envVars.Keys) { $psi.EnvironmentVariables[$k]=$envVars[$k] }

$proc=[System.Diagnostics.Process]::Start($psi)
Set-Content $pidPath $proc.Id
Write-Output "Host iniciado: PID $($proc.Id), API 127.0.0.1:8765."
