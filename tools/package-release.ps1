param([switch]$VisualOnlySinceValidation)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$version='0.3.0'
$build=Join-Path $root 'firmware/esp32-c6/build'
$output=Join-Path $root ".artifacts/tokenbuddy-$version"
$archive="$output.zip"
if ((Test-Path $output) -or (Test-Path $archive)) {throw 'Pacote existente; preservar artefatos anteriores.'}
$dirty=git -c "safe.directory=$root" -C $root status --porcelain
if ($LASTEXITCODE -ne 0 -or $dirty) {throw 'Commitar fontes antes de empacotar.'}
$sdkVersion=git -C "$root/.tools/esp-idf-v5.5.2" describe --tags --dirty
if ($LASTEXITCODE -ne 0 -or $sdkVersion -ne 'v5.5.2') {throw 'SDK nao corresponde a tag limpa validada.'}
$binary=Get-Item "$build/tokenbuddy_device_os.bin"
$hardware=Get-Item "$root/.tools/integrated-hardware-report.json"
$recovery=Get-Item "$root/.tools/recovery-hardware-report.json"
foreach ($reportFile in @($hardware,$recovery)) {
    $report=Get-Content -Raw $reportFile.FullName | ConvertFrom-Json
    if ($report.result -ne 'PASS') {throw 'Relatorio de validacao nao aprovado.'}
    if ($reportFile.LastWriteTimeUtc -lt $binary.LastWriteTimeUtc -and -not $VisualOnlySinceValidation) {throw 'Validacao atualizada obrigatoria; revisao somente visual exige declaracao explicita.'}
}
if ($VisualOnlySinceValidation) {
    $baseline=Get-Content -Raw "$root/.tools/validated-base-0.3.0/release-validation-summary.json" | ConvertFrom-Json
    $baselineFile=Get-Item "$root/.tools/validated-base-0.3.0/tokenbuddy_device_os.bin"
    if ($baseline.result -ne 'PASS' -or (Get-FileHash $baselineFile.FullName -Algorithm SHA256).Hash -ne $baseline.sha256) {throw 'Baseline validada nao confirmada.'}
}
New-Item -ItemType Directory -Force "$output/firmware/bootloader","$output/firmware/partition_table","$output/validation","$output/source" | Out-Null
Copy-Item -LiteralPath $binary.FullName -Destination "$output/firmware/tokenbuddy_device_os.bin"
Copy-Item "$build/bootloader/bootloader.bin" "$output/firmware/bootloader/bootloader.bin"
Copy-Item "$build/partition_table/partition-table.bin" "$output/firmware/partition_table/partition-table.bin"
Copy-Item "$build/flasher_args.json","$build/flash_args" "$output/firmware"
Copy-Item $hardware.FullName "$output/validation/hardware.json"
Copy-Item $recovery.FullName "$output/validation/recovery.json"
if ($VisualOnlySinceValidation) {
    Copy-Item $baselineFile.FullName "$output/validation/functional-baseline.bin"
    Copy-Item "$root/.tools/validated-base-0.3.0/release-validation-summary.json" "$output/validation/functional-baseline-summary.json"
    'Apos a baseline funcional validada, o usuario solicitou refinamento visual sem repetir validacao. Imagem atual: build/flash verificados; teste longo refere-se a baseline, nao a revisao visual.' | Set-Content -Encoding UTF8 "$output/validation/visual-revision.txt"
}
$tracked=git -c "safe.directory=$root" -C $root ls-files
if ($LASTEXITCODE -ne 0) {throw 'Falha listando fontes versionadas.'}
foreach ($relative in $tracked) {
    $destination=Join-Path "$output/source" $relative
    New-Item -ItemType Directory -Force (Split-Path $destination -Parent) | Out-Null
    Copy-Item -LiteralPath (Join-Path $root $relative) -Destination $destination
}
$commit=git -c "safe.directory=$root" -C $root rev-parse HEAD
$files=Get-ChildItem "$output/firmware" -File -Recurse | ForEach-Object {
    @{path=$_.FullName.Substring($output.Length+1).Replace('\','/');bytes=$_.Length;sha256=(Get-FileHash $_.FullName -Algorithm SHA256).Hash}
}
@{name='TokenBuddy Device OS';version=$version;source_commit=$commit;sdk=$sdkVersion;
    board='ESP32-C6-LCD-1.47 non-touch; validated FH8/8MB unit';flash_mb=8;files=@($files);
    hardware_validation= $(if ($VisualOnlySinceValidation) {'FUNCTIONAL_BASELINE_PASS_VISUAL_REVISION_NOT_RETESTED'} else {'PASS'});
    recovery_validation='PASS_ON_FUNCTIONAL_BASELINE';visual_revision=[bool]$VisualOnlySinceValidation} | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 "$output/manifest.json"
@'
TokenBuddy Device OS 0.3.0 — MVP USB

Firmware validado na unidade Waveshare ESP32-C6-LCD-1.47 sem touch, FH8/8MB.
Nao usar na variante Touch/JD9853 ou presumir compatibilidade com unidade 4MB.
Conferir chip, MAC e porta antes de qualquer gravacao. Fontes e instrucoes em
source/README.md; protocolo e uso em source/docs/host.md e source/docs/protocol.md.

Este pacote nao inclui SDK/toolchain, dependencias Python, credenciais, chave
local da API nem backup do firmware original. O backup permanece na workstation.
Validacao e limitada aos testes registrados; nao e certificacao de produto.
'@ | Set-Content -Encoding UTF8 "$output/README.txt"
Compress-Archive -Path "$output/*" -DestinationPath $archive
$hash=(Get-FileHash $archive -Algorithm SHA256).Hash
"$hash  $([IO.Path]::GetFileName($archive))" | Set-Content -Encoding UTF8 "$archive.sha256"
Write-Output "Pacote criado: $archive"
Write-Output "SHA256: $hash"
