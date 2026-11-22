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

Primeiro flash autorizado concluído na COM3; hashes verificados e quatro heartbeats
TokenBuddy recebidos, heap livre constante de 464108 bytes em cerca de 20 s.
Backup completo anterior preservado localmente. Veja [validação](docs/hardware-validation.md)
e [referência visual](docs/ui-reference.md). LCD próprio ainda não inicializado.

LCD validado em hardware: teste de cinco cores, borda e orientação confirmados
pelo usuário; logs com heap estável durante captura de 25 segundos.
Firmware atual é o teste LCD. Próximo marco: avatar animado.
Veja [evidências do LCD](docs/lcd-validation.md).
