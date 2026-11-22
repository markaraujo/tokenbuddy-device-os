# Arquitetura — TokenBuddy Device OS 0.3.0

Aplicação ESP-IDF/FreeRTOS, não um sistema operacional completo. Unidade alvo:
Waveshare ESP32-C6-LCD-1.47 sem touch, flash física de 8 MB.

```text
Host Windows: FastAPI + métricas + conectores de processos/Codex JSONL
                         |
                   USB JSONL v1
                         |
connectivity -> control/model -> behavior -> pose -> ui/avatar -> display -> board
                     ^                                           LVGL      SPI/RGB
                     |
                BOOT GPIO9
```

- board: pinagem validada, SPI, ST7789T, backlight, RGB e leitura BOOT.
- display: buffers estáticos, LVGL e callback de transferência SPI.
- behavior: C puro, prioridades, expiração, estado base, piscadas e olhar.
- control: modelo de telas, eventos, debounce e protocolo validado; testável no host.
- avatar/ui: desenho e cinco telas, sem criar objetos por frame.
- connectivity: transporte USB com buffers e trabalho por iteração limitados.
- main: coordenação, NVS de ajustes, diagnóstico e teste de boot.

Uma única tarefa altera LVGL e o modelo. Tick de 2 ms via esp_timer; callback SPI
somente libera o buffer. Loop principal aproximadamente 20 ms; não há promessa
de FPS medido. Buffers LCD 2 x 172 x 20 pixels; RX máximo 768 bytes, profundidade
JSON 8; eventos em anel de quatro posições. Erros não aplicam mudanças parciais.

Host: um worker possui a serial e fila de 32 comandos. ACK confirma comando;
heartbeat/telemetria independentes de tráfego de agentes. Toda conexão confirma
VID/PID, MAC e fw antes de comandos. API loopback autenticada, sem credenciais no
ESP32 e sem abrir portas de firewall. MVP acompanha uma tarefa ativa por vez.

Dependências fixadas: ESP-IDF 5.5.2 limpo, LVGL 8.3.11, led_strip 2.5.5;
Python host com lock. Driver ST7789T oficial preservado com licença e procedência.
Testes C rodam nativamente no Windows e no boot; testes Python cobrem API, serial,
conectores e indisponibilidade. Testes físicos cobrem flash e troca real de dados.

[Contrato](protocol.md), [host](host.md), [decisão USB](decisions/0001-usb-mvp.md).
Nenhum módulo futuro vazio, credencial Wi-Fi, LLM no ESP32 ou driver de sensor
não instalado foi adicionado. Wi-Fi, IMU, touch externo, voz e OTA são evoluções
separadas; o transporte USB permite validar o produto disponível hoje.
