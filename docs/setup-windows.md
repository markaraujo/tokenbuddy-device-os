# Windows — ambiente validado

Workspace atual: `D:\Avatar_Token`. Python e ferramentas globais existentes foram
preservados; SDK/toolchain/ambiente host estão em `.tools` dentro do projeto.

| Item | Versão/caminho |
|---|---|
| Windows 11 Pro | 10.0.26300, 64 bits |
| Git | 2.52.0.windows.1 |
| Python | 3.13.14, instalação existente preservada |
| VS Code / extensão Espressif | 1.147.0 / 2.3.0 |
| Codex CLI | 0.171.0 |
| ESP-IDF | tag limpa v5.5.2, commit 30aaf64524299d3bde422ca9a2848090d1bc5d0f |
| GCC RISC-V | 14.2.0, esp-14.2.0_20251107 |
| CMake / Ninja | 3.30.2 / 1.12.1 |
| esptool | 4.14.0 |
| LVGL / led_strip | 8.3.11 / 2.5.5, manifests e lock versionados |
| Python SDK | .tools/espressif/python_env/idf5.5_py3.13_env/Scripts/python.exe |
| Rust host | services/host/target/release/tokenbuddy-server.exe (cargo build --release) |
| TinyCC host | 0.9.27, .tools/tinycc/tcc/tcc.exe |

## Terminal e build

```powershell
. ./tools/activate.ps1
idf.py --version
./tools/idf.ps1 build
./tools/idf.ps1 size
./tools/test.ps1
```

activate define IDF_PATH, IDF_TOOLS_PATH e IDF_TARGET apenas para o processo e
executa export.ps1 do SDK. Wrappers validam exit code nativo; stderr informativo
nao deve ser confundido com falha pelo Windows PowerShell ao redirecionar logs.
Nenhum clean build é usado como recuperação genérica.

VS Code: abrir o workspace; tarefa padrão de build e tarefas de testes/start/stop
host preparadas. Perfil de terminal TokenBuddy ESP-IDF executa activate.ps1.
compile_commands.json é gerado pelo build e apontado pelo workspace. Não foram
sobrescritas configurações globais do editor. Os scripts foram executados e os
JSONs verificados; a seleção EIM nativa da extensão não foi operada na interface.
As opções antigas idf.espIdfPath/idf.toolsPath foram removidas pela extensão 2.x;
o fluxo validado usa as tarefas locais, sem depender dessa seleção global.

## Reprodução da instalação

```powershell
$env:IDF_TOOLS_PATH = "$PWD/.tools/espressif"
git clone --branch v5.5.2 --depth 1 --shallow-submodules --recursive https://github.com/espressif/esp-idf.git .tools/esp-idf-v5.5.2
& ./.tools/esp-idf-v5.5.2/install.ps1 esp32c6
. ./tools/activate.ps1
./tools/host.ps1 setup
```

TinyCC foi obtido do servidor de releases indicado pelo site do projeto;
procedência/hash e comando de teste em avatar-mvp.md. O host Rust usa o
workspace em services/host (build: `./tools/host.ps1 setup`).

## Gravação deliberada

Parar o host antes de usar a COM3. Conferir nome/revisão física da placa.

```powershell
./tools/host.ps1 stop
./tools/flash.ps1 -Port COM3 -ExpectedMac 14:c1:9f:05:7b:ec -Confirmed
./tools/host.ps1 start
```

O script exige confirmação explícita, tag limpa, ESP32-C6, MAC e flash 8 MB;
nenhum erase_flash completo, gravação de eFuse ou mudança de driver é executado.
O firmware atual destina-se à unidade sem touch de 8 MB validada, não à variante
Touch/JD9853 nem automaticamente às versões com flash de 4 MB.

## Evidências e histórico de recuperação

Build mínimo, demo Waveshare e firmware integrado compilaram. Flash, logs e LCD
foram testados no dispositivo; relatórios em .tools e documentação de validação.

A instalação inicialmente encontrou DNS intermitente no roteador 192.168.3.1.
Após autorização, somente Ethernet (índice 14) passou a usar 1.1.1.1/1.0.0.1;
downloads voltaram a funcionar. Snapshot em .tools/dns-before.json. Reversão:
`Set-DnsClientServerAddress -InterfaceIndex 14 -ResetServerAddresses` (DHCP), ou
`-ServerAddresses 192.168.3.1` para restaurar explicitamente o servidor observado.
Isso exige PowerShell administrativo; nenhuma outra interface foi alterada.

A instalação SDK tinha uma diferença local em LOG_COMPRESS_SRCS. Antes da
restauração, conteúdo e diff foram preservados em .tools/sdk-local-CMakeLists-preserved.txt
e .tools/sdk-local-diff-before-restore.patch. Somente esse arquivo foi restaurado
à tag, sem limpar build. SDK agora limpo; firmware e demo oficial recompilaram.
Origem da diferença inicial não foi determinada. A validação do SDK refere-se à
combinação desta placa, driver e versões testadas, não a todas as APIs do SDK.

Git já estava inicializado pelo usuário da sandbox. Comandos usam exceção de
safe.directory apenas no processo, sem alterar configuração global.

Fontes:
- https://docs.espressif.com/projects/esp-idf/en/v5.5.2/esp32c6/get-started/windows-setup.html
- https://github.com/espressif/esp-idf/tree/v5.5.2
- https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47
