# TokenBuddy USB protocol v1

UTF-8 JSON por linha (NDJSON), 115200 baud, USB Serial/JTAG. Firmware recebe
no máximo 768 bytes antes de LF e rejeita profundidade JSON maior que 8.
Após uma linha inválida/longa, o próximo LF encerra o descarte e permite retomar.
Strings de agente: ASCII imprimível, 1–31 caracteres. Não há comandos de shell,
erase, gravação de eFuse ou acesso a arquivos pelo protocolo.

Toda solicitação usa v=1 e id inteiro entre 0 e 2147483647. Exemplo:

```json
{"v":1,"id":1,"type":"heartbeat"}
{"v":1,"type":"ack","id":1,"ok":true,"error":""}
```

| type | Campos | Efeito |
|---|---|---|
| heartbeat | nenhum | Host presente; TTL de 5 segundos |
| telemetry | cpu, ram, ram_used, ram_total | CPU/RAM reais; percentuais 0–100; memória em MiB |
| telemetry (GPU opcional) | gpu, vram_used, vram_total | GPU 0–100; VRAM em MiB; omitir quando indisponível |
| navigate | page: 0–4 | Avatar, Agentes, Sistema, Eventos, Ajustes |
| settings | brightness: 0–50; animation: 0–2; demo: bool | Campos opcionais; alterações validadas atomicamente |
| expression | state: 0–9; duration: 50–60000 ms | Expressão temporária, respeita prioridade |
| event | event, agent; input_tokens/output_tokens opcionais | Eventos reais de tarefa; tokens precisam vir em par |

Eventos: agent.started, agent.completed, agent.failed. Estados numéricos:
0 idle, 1 happy, 2 thinking, 3 error, 4 curious, 5 working, 6 success,
7 sleeping, 8 disconnected, 9 dizzy. O estado base é WORKING durante uma tarefa,
IDLE quando online sem tarefa e DISCONNECTED após expirar o heartbeat.
Sucesso/erro são temporários (5 s). Erro tem prioridade maior que sucesso e trabalho.

Um comando inválido não altera parcialmente o modelo. Erros: invalid_json,
too_long, too_deep, invalid_command, unknown_type e priority. O transporte pode
reportar id=-1 quando uma linha não permite recuperar um identificador.
Campos desconhecidos são ignorados; versões e tipos desconhecidos são rejeitados.

O firmware envia status a cada segundo: identidade MAC-48, fw, uptime_ms, motivo
do reset (enum esp_reset_reason do SDK), página, estado, online, brilho, animação,
demo, heap do sistema, lv_free/lv_frag, margem mínima da stack principal,
contadores de BOOT curto/longo, settings_saved e selftest.
A MAC-48 é obtida por esp_read_mac(ESP_MAC_WIFI_STA), sem ligar o rádio. A API
de eFuse no C6 fornece EUI-64 de 8 bytes e não deve receber buffer de 6 bytes.

Não existem retries automáticos de eventos após timeout: o resultado pode ser
incerto se o ACK foi perdido. Reconexão verifica VID/PID, MAC e versão antes de
enviar qualquer comando, e só restaura uma tarefa ainda marcada como ativa.
A API host possui task_id para impedir que uma segunda tarefa conclua a primeira;
o MVP acompanha uma tarefa ativa por vez. Reset não mantém tarefas como ativas
sem um host que as confirme novamente.

Brilho/animação são persistidos em NVS. Se NVS falhar, o firmware funciona em RAM
e informa settings_saved=false; não apaga partições para tentar corrigir isso.
