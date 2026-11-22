# TokenBuddy AI

TokenBuddy Device OS 0.2.0 para Waveshare ESP32-C6-LCD-1.47 sem touch.
Workspace: `D:\Avatar_Token`.

Avatar MVP gravado na COM3: olhos ciano, piscadas, movimento do olhar e ciclo local
repouso/feliz/concentrado/erro, com RGB integrado. Não há conexão com agentes.
LCD e pinagem validados na etapa anterior. Backup original de 8 MB preservado
localmente. SDK candidato continua 5.5.2-dirty; homologação limpa pendente.

```powershell
./tools/test-behavior.ps1
. ./tools/activate.ps1
./tools/idf.ps1 build
./tools/idf.ps1 size
```

VS Code: tarefa padrão de build ou perfil de terminal `TokenBuddy ESP-IDF`.
Nunca gravar sem reconfirmar dispositivo e autorização. Não montar/formatar microSD.

- [Avatar, testes e limitações](docs/avatar-mvp.md)
- [Arquitetura](docs/architecture.md)
- [Validação LCD](docs/lcd-validation.md)
- [Setup Windows](docs/setup-windows.md)
- [Referência visual](docs/ui-reference.md)
