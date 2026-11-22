param(
    [Parameter(Mandatory=$true)][ValidatePattern('^COM[0-9]+$')][string]$Port,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-fA-F]{2}(:[0-9a-fA-F]{2}){5}$')][string]$ExpectedMac,
    [switch]$Confirmed
)
$ErrorActionPreference='Stop'
if (-not $Confirmed) {throw 'Confirme explicitamente a gravacao com -Confirmed apos conferir placa e porta.'}
$root=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/activate.ps1"
$sdkVersion=git -C $env:IDF_PATH describe --tags --dirty
if ($LASTEXITCODE -ne 0 -or $sdkVersion -ne 'v5.5.2') {throw 'SDK diverge da tag limpa v5.5.2.'}
$previous=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    $identity=& python -m esptool --chip esp32c6 --port $Port read_mac 2>&1
    $identityExit=$LASTEXITCODE
    if ($identityExit -ne 0 -or -not ($identity | Select-String -SimpleMatch -Pattern "BASE MAC: $($ExpectedMac.ToLower())")) {throw 'Chip/MAC nao confirmado; gravacao cancelada.'}
    $flashIdentity=& python -m esptool --chip esp32c6 --port $Port flash_id 2>&1
    if ($LASTEXITCODE -ne 0 -or -not ($flashIdentity | Select-String -SimpleMatch -Pattern 'Detected flash size: 8MB')) {throw 'Esta versao exige a unidade de 8 MB validada.'}
    & python "$env:IDF_PATH/tools/idf.py" -C "$root/firmware/esp32-c6" -p $Port flash
    $flashExit=$LASTEXITCODE
} finally {$ErrorActionPreference=$previous}
if ($flashExit -ne 0) {throw "Flash falhou: $flashExit"}
