# TokenBuddy AI

TokenBuddy Device OS para Waveshare ESP32-C6-LCD-1.47 sem touch.
Workspace: `D:\Avatar_Token`.

Ambiente local instalado; firmware mínimo compilado para ESP32-C6 (147456 bytes).
COM3 identificada como ESP32-C6FH8 com flash 8 MB. Nenhum flash realizado.
SDK candidato v5.5.2-dirty; diferença local preservada para revisão antes de homologar.

```powershell
. ./tools/activate.ps1
./tools/idf.ps1 build
./tools/idf.ps1 size
```

VS Code: tarefa padrão de build ou perfil de terminal `TokenBuddy ESP-IDF`.
Consulte [setup Windows](docs/setup-windows.md), [hardware](docs/hardware.md) e [plano](docs/development-plan.md).
