# Arquitetura atual

TokenBuddy Device OS 0.2.0: aplicação ESP-IDF/FreeRTOS para unidade Waveshare
ESP32-C6-LCD-1.47 sem touch, flash física 8 MB.

Fluxo: main coordena behavior -> pose -> avatar/LVGL -> display -> board/ST7789T.
O board também controla o RGB integrado. Não há dependência de behavior para
hardware: ele usa somente C padrão, com testes executáveis no host.

Uma tarefa manipula LVGL. Tick de 2 ms via esp_timer; callback SPI só libera o
buffer de desenho. Buffers fixos e objetos criados na inicialização; estados
atualizam desenho e texto. Não são criados objetos LVGL por frame.

Modo atual: demonstração local de quatro estados a cada 6 s, sem eventos reais
de agentes, rede ou métricas. A prioridade protege ERROR de HAPPY/THINKING até
expirar; IDLE é o estado de retorno. Piscadas e olhar independem do frame rate.

Dependências fixadas: LVGL 8.3.11, led_strip 2.5.5 e SDK candidato 5.5.2-dirty.
Driver ST7789T oficial Waveshare preservado com licença. A diferença local do SDK
continua pendente de homologação, conforme setup-windows.md.

Próximas capacidades só serão implementadas quando necessárias: eventos externos,
entradas físicas disponíveis e conectividade. Não existem módulos futuros vazios.
