# Setup Windows — validado em 2026-11-22

Workspace: `D:\Avatar_Token`.

| Ferramenta | Versão |
|---|---|
| Windows 11 Pro | 10.0.26300, 64 bits |
| Git | 2.52.0.windows.1 |
| Python existente e ambiente IDF | 3.13.14 |
| VS Code / extensão Espressif | 1.147.0 / 2.3.0 |
| Codex CLI | 0.171.0 |
| ESP-IDF | v5.5.2-dirty, commit 30aaf64524299d3bde422ca9a2848090d1bc5d0f |
| GCC RISC-V | 14.2.0, esp-14.2.0_20251107 |
| CMake / Ninja | 3.30.2 / 1.12.1 |
| esptool | 4.14.0 |

SDK: `.tools/esp-idf-v5.5.2`. Ferramentas: `.tools/espressif`.
Python gerenciado: `.tools/espressif/python_env/idf5.5_py3.13_env/Scripts/python.exe`.
Python original preservado. Instalação das ferramentas e Python terminou com exit 0;
export.ps1 confirmou dependências OK. Submódulos completos, sem revisões divergentes.

## Comandos

```powershell
. ./tools/activate.ps1
idf.py --version
./tools/idf.ps1 build
./tools/idf.ps1 size
```

Build mínimo: exit 0, 1062 etapas, binário de 147456 bytes (0x24000),
86% da partição de aplicação de 1 MB livre. `size`: exit 0.
Artefato: `firmware/esp32-c6/build/tokenbuddy_device_os.bin`.
Logs de instalação, build, tamanho e consultas do chip estão em `.tools/`.

Causa do erro inicial de size: Windows PowerShell interpretava stderr informativo
como exceção por ErrorActionPreference=Stop. Wrapper corrigido para validar exit code;
size repetido com sucesso, sem clean ou recompilação completa.

## VS Code

Perfil `TokenBuddy ESP-IDF` e tarefa padrão de build usam ativação explícita local.
JSON e sintaxe PowerShell validados. compile_commands.json foi gerado pelo build.
A extensão 2.3.0 removeu os antigos parâmetros idf.espIdfPath/idf.toolsPath; seleção
nativa via EIM ainda não validada na interface. Build no terminal está validado.

## DNS: resolvido com autorização

Causa provável: encaminhamento DNS intermitente no roteador 192.168.3.1.
Consultas padrão expiravam; consulta direta a 1.1.1.1 funcionava. Após autorização,
Ethernet (índice 14) foi configurada com 1.1.1.1 e 1.0.0.1. Consultas padrão,
downloads do SDK, ferramentas, pacotes Python e demo Waveshare voltaram a funcionar.
Snapshot anterior: `.tools/dns-before.json`.

Reversão, em PowerShell administrativo:
`Set-DnsClientServerAddress -InterfaceIndex 14 -ResetServerAddresses` (DHCP),
ou `-ServerAddresses 192.168.3.1` para restaurar explicitamente o servidor observado.
Nenhuma alteração de drivers, PATH global ou outras interfaces.

## Hardware identificado, sem flash

COM3: Espressif USB Serial/JTAG, VID 303A/PID 1001; MAC base 14:c1:9f:05:7b:ec.
esptool identificou ESP32-C6FH8, revisão v0.2, cristal 40 MHz.
flash_id confirmou 8 MB, fabricante 20, device 4017. Consultas tiveram exit 0 e
reiniciaram a placa; não gravaram firmware persistente.

A capacidade real diverge dos 4 MB previstos para o modelo. Manter configuração
conservadora de 4 MB no firmware mínimo até confirmar a revisão física da placa.
Não inferir modelo Waveshare apenas pelo chip. Flash e teste de boot pendentes.
Após confirmação explícita da placa, porta e gravação:

```powershell
. ./tools/activate.ps1
Push-Location firmware/esp32-c6
idf.py -p COM3 flash monitor
Pop-Location
```

## Limitações antes de homologar o SDK

Foi detectada diferença local em
`components/bt/common/ble_log/extension/log_compression/CMakeLists.txt`:
adição de ble_log_compression.c a LOG_COMPRESS_SRCS. Origem não determinada.
Nenhuma correção automática aplicada; diff preservado em `.tools/sdk-local-diff.patch`.
O build passou com esse SDK, mas v5.5.2 oficial intacto e demo LCD ainda não estão
homologados. Revisar essa diferença antes de aprovar a base para produção.

Demo oficial baixado em `.tools/waveshare-demo`; LVGL incluído 8.3.11,
ST7789T customizado, SPI 12 MHz e offset X=34/Y=0. Não compilado nem gravado.

Fontes oficiais utilizadas:
- https://docs.espressif.com/projects/esp-idf/en/v5.5.2/esp32c6/get-started/windows-setup.html
- https://github.com/espressif/esp-idf/tree/v5.5.2
- https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47
