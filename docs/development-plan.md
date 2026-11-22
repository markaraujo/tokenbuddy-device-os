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
