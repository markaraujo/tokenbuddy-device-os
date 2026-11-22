# Arquitetura incremental

Aplicação ESP-IDF/FreeRTOS, inicialmente apenas boot e heartbeat com heap livre no console USB Serial/JTAG.
Sem rede, credenciais ou inicialização de GPIOs nesta etapa.
Próxima separação: board (SPI/LCD/backlight), display (LVGL), avatar (expressões), behavior (eventos e prioridades).
LVGL ainda não selecionado: fixar versão após revisar o demo oficial; não misturar APIs v8/v9.
Eventos físicos e do host alimentarão lógica local independente de hardware. Protocolo de rede será decidido na Fase 5.
