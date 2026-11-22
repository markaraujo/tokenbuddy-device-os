# ST7789T Waveshare

Fontes preservadas de ESP32-C6-LCD-1.47-Demo.zip, diretório
ESP-IDF/ESP32-C6-LCD-1.47-Test/main/LCD_Driver/Vernon_ST7789T.
Licença Apache-2.0 indicada nos arquivos; cabeçalhos de autoria preservados.
Origem: https://files.waveshare.com/wiki/ESP32-C6-LCD-1.47/ESP32-C6-LCD-1.47-Demo.zip
SHA256 do ZIP: 4B960E8CCD58E54E60494A2A6DA62531695C9152C9950153CAD34ACC496C4EE4.

O board.c usa a sequência validada no LCD: 12 MHz, BGR, espelhamento horizontal,
gap 34/0. GPIO8 RGB conferido no esquema sem touch e foto da placa.
Não usar este BSP para a variante Touch/JD9853.

As chamadas de comandos de inicialização do fornecedor nem sempre checam retorno.
Teste visual e logs continuam necessários; não há leitura de ID do painel via SPI.
