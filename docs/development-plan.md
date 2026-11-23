# Etapas e critérios

| Etapa | Situação atual |
|---|---|
| 0 — Ambiente | SDK 5.5.2 limpo, demo oficial e firmware compilam; scripts/tarefas VS Code preparados |
| 1 — Placa | ESP32-C6FH8/8 MB identificado; flash e execução real validados |
| 2 — LCD | Cinco cores/borda/orientação confirmadas pelo usuário |
| 3 — Avatar | MVP 0.3.0: dez estados, animações, RGB, cinco telas e BOOT |
| 4 — Entradas | BOOT implementado/testado; MPR121 e IMU externos ainda indisponíveis |
| 5 — Comunicação | USB/heartbeat/offline/API local implementados; Wi-Fi/provisionamento futuros |
| 6 — Agentes | Processos reais e adaptador Codex JSONL; outros agentes futuros |

Critérios do MVP com hardware disponível atendidos: build/flash, funcionamento
local, entrada real, telas, telemetria real, eventos e protocolo testados; ajustes
persistem após reset, erros são rejeitados, logs e pacote reproduzíveis.

A entrega cobre o MVP de hoje, não todas as ambições do produto. Antes de adicionar
Wi-Fi, sensores ou áudio, conferir componentes, alimentação, pinagem e requisitos.
Não criar drivers vazios nem presumir que a variante sem touch tem QMI8658.

Próximas fases exigem decisão/recursos concretos: componentes físicos, configuração
de rede, provedor/modelo/privacidade de voz e memória, e requisitos de dashboard/OTA.
Primeira evolução recomendada: observar uso diário e estabilidade de longa duração;
depois integrar MPR121/IMU quando os módulos estiverem disponíveis.

Evidências e limitações completas: [release 0.3.0](release-0.3.0.md).

## Registro de recuperação (2026-11-22)

- Instabilidade observada na revisão de legibilidade (reset espontâneo em janela
  curta; firmware parou de emitir status após o reset — boot hang, não watchdog limpo).
- Causa provável registrada: a revisão de legibilidade nunca passou por soak antes
  da instabilidade aparecer; isolamento feito gravando a base validada 0.3.0
  (SHA256 `8AF2AC...F965`) na mesma unidade.
- Base revalidada após recuperação: `test.ps1` PASS + `test-hardware.py --soak-seconds 180`
  PASS (9/9, 90 amostras). Observação de longa duração em curso via
  `tools/stability-observe.py` (relatório em `.tools/stability-report.jsonl`).
- Nota de hardware: ESP32-C6 USB-Serial-JTAG não responde a reset por DTR/RTS;
  quando o firmware fica mudo, exige reboot físico para recuperar.

## Teste ao vivo Codex (2026-11-22)

- `tokenbuddy codex -- "responda apenas ok"` executou sessão real (`codex-cli 0.171.0`):
  eventos oficiais `thread.started` → `turn.started` → `item.completed` →
  `turn.completed` com usage real (18152 input / 5 output tokens).
- Dispositivo reagiu: estado passou de 8 (disconnected/online-false) para
  **6 = success** após o `turn.completed`; host seguiu connected/fresh, sem erros.
- Causa provável registrada (não corrigido): exit code 1 no PowerShell vem de
  stderr do codex (linha "Reading additional input from stdin..." + erro rmcp
  de transporte para localhost:9100) sendo tratado como NativeCommandError;
  o JSONL de eventos e o uso da conta foram normais. Observar se persiste em
  PowerShell com `$ErrorActionPreference='Continue'` já configurado no CLI.
