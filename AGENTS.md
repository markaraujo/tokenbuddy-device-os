# TokenBuddy AI

- Firmware ESP-IDF para Waveshare ESP32-C6-LCD-1.47 sem touch.
- Trabalhar em fases: build mínimo, identificação/flash, LCD oficial, avatar.
- Versão candidata do SDK: v5.5.2; validar instalação e exemplo Waveshare antes de homologar.
- Build: `./tools/idf.ps1 build`. Ambiente: IDF_PATH e IDF_TOOLS_PATH locais; executar export.ps1 do SDK primeiro.
- Lógica futura de comportamento deve ser independente do hardware e testável no host.
- Não atribuir GPIOs sem conferir esquema e conflitos com microSD/USB.
- Nunca gravar sem identificar chip/porta e obter confirmação explícita do usuário.
- Não apagar arquivos, alterar drivers ou configurações globais sem autorização.
- Registrar causa provável antes de corrigir; máximo de duas tentativas sem evidência nova. Não limpar build como solução genérica.
- Não declarar build, flash ou teste físico sem resultado real. Não adicionar módulos futuros vazios.

- Arquitetura atual: board -> display/LVGL; behavior em C puro -> avatar; main coordena a demo local.
- Testes host: `./tools/test-behavior.ps1` (TinyCC portátil local; docs/avatar-mvp.md).
- Dependências fixadas: LVGL 8.3.11 e led_strip 2.5.5, registrar dependencies.lock.
