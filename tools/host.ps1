param([ValidateSet('setup','start','stop','status')][string]$Action='status')
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$python=Join-Path $root '.tools/host-env/Scripts/python.exe'
$pidPath=Join-Path $root '.tools/host.pid'
$configPath=Join-Path $root '.tools/host-config.json'
if ($Action -eq 'setup') {
    if (-not (Test-Path $python)) { & python -m venv "$root/.tools/host-env"; if ($LASTEXITCODE -ne 0) {throw 'venv falhou'} }
    $previousPreference=$ErrorActionPreference
    try {
        $ErrorActionPreference='Continue'
        if (Test-Path "$root/services/core/requirements.lock") {
            & $python -m pip install --disable-pip-version-check -r "$root/services/core/requirements.lock"
            if ($LASTEXITCODE -ne 0) {throw 'Dependencias host falharam'}
        }
        & $python -m pip install --disable-pip-version-check -e "$root/services/core[test]"
        $installExit=$LASTEXITCODE
    } finally { $ErrorActionPreference=$previousPreference }
    if ($installExit -ne 0) {throw 'Instalacao host falhou'}
    if (-not (Test-Path $configPath)) {
        $randomBytes=New-Object byte[] 32
        $generator=[Security.Cryptography.RandomNumberGenerator]::Create()
        try { $generator.GetBytes($randomBytes) } finally { $generator.Dispose() }
        @{port='COM3';mac='14:c1:9f:05:7b:ec';http_port=8765;api_key=[Convert]::ToBase64String($randomBytes)} | ConvertTo-Json | Set-Content -Encoding UTF8 $configPath
    }
    exit 0
}
if (-not (Test-Path $python) -or -not (Test-Path $configPath)) {throw 'Execute ./tools/host.ps1 setup primeiro.'}
if ($Action -eq 'status') { & $python -m tokenbuddy status; exit $LASTEXITCODE }
$existing=$null
if (Test-Path $pidPath) {
    $savedPid=Get-Content $pidPath -Raw
    if ($savedPid.Trim() -match '^\d+$' -and [long]$savedPid.Trim() -gt 0) { $existing=Get-CimInstance Win32_Process -Filter "ProcessId=$($savedPid.Trim())" }
}
if ($existing -and ($existing.CommandLine -notlike '*tokenbuddy serve*' -or $existing.ExecutablePath -notlike "$root\.tools\host-env\*")) {throw 'PID existente nao corresponde ao host; nenhuma acao executada.'}
if ($Action -eq 'stop') {
    if ($existing) {
        $children=Get-CimInstance Win32_Process -Filter "ParentProcessId=$($existing.ProcessId)"
        foreach ($child in $children) {if ($child.Name -ne 'conhost.exe' -and $child.CommandLine -notlike '*tokenbuddy serve*') {throw 'Filho inesperado; interrupcao cancelada.'}}
        foreach ($child in $children) {if ($child.Name -ne 'conhost.exe') {Stop-Process -Id $child.ProcessId -ErrorAction SilentlyContinue}}
        Stop-Process -Id $existing.ProcessId -ErrorAction SilentlyContinue
        'Host encerrado.'
    }
    Set-Content $pidPath '0'
    exit 0
}
if ($existing) {Write-Output 'Host ja esta ativo.';exit 0}
$process=Start-Process -FilePath $python -ArgumentList '-m','tokenbuddy','serve' -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput "$root/.tools/host.stdout.log" -RedirectStandardError "$root/.tools/host.stderr.log"
Set-Content $pidPath $process.Id
Write-Output "Host iniciado: PID $($process.Id), API 127.0.0.1:8765."
