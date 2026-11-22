# Hardware validado

Unidade fotografada: **Waveshare ESP32-C6-LCD-1.47, sem touch**.
[Foto do verso](reference/board-back.jpeg). Inscrição identificada; revisão de PCB
não legível. Chip real ESP32-C6FH8 QFN32, revisão v0.2, cristal 40 MHz, flash 8 MB,
confirmados por esptool. MAC base 14:c1:9f:05:7b:ec, COM3, USB VID303A/PID1001.

O esquema publicado especifica FH4/4 MB, mas o teste físico de cinco cores,
borda e orientação confirmou o driver e a pinagem sem touch nesta unidade FH8.
Não generalizar o resultado para outras revisões. A variante Touch usa JD9853 e
outra pinagem; sua IMU/touch não são presumidos presentes nesta placa.

| Sinal | GPIO | Uso atual |
|---|---|---|
| LCD MOSI | 6 | Compartilhado fisicamente com microSD |
| LCD SCLK | 7 | Compartilhado fisicamente com microSD |
| LCD CS | 14 | SPI 12 MHz |
| LCD DC | 15 | Comando/dados |
| LCD RESET | 21 | Reset do painel |
| Backlight | 22 | PWM 5 kHz, inicial 30%, máximo 50% |
| RGB WS2812 | 8 | RMT, brilho baixo |
| BOOT | 9 | Entrada com pull-up, debounce 30 ms, longo 800 ms |
| microSD CS | 4 | Mantido alto, nunca montado/formatado |
| microSD MISO | 5 | Não inicializado pelo firmware |
| USB D-/D+ | 12/13 | Reservados ao USB; não alocar |

LCD ST7789T, 172 x 320, RGB565/BGR, espelhamento horizontal e offset X34/Y0,
baseado no driver oficial e validado visualmente pelo usuário.

BOOT curto: próxima tela. BOOT longo no avatar: alternar demo local; em Ajustes:
próximo nível de brilho (0–50%); nas demais telas: voltar ao avatar. RESET mantém
sua função física. BOOT mantido durante RESET pode entrar no bootloader ROM;
isso é comportamento da placa, não navegação do aplicativo.

O esquema sem touch não possui acelerômetro/IMU integrado. MPR121, QMI8658/BMI270,
áudio, bateria e LEDs externos continuam componentes futuros não presumidos
instalados. Não atribuir pinos a eles sem revisar esquema, conflitos e alimentação.

Esquema preservado em .tools/waveshare-schematic.pdf. Fonte:
https://files.waveshare.com/wiki/ESP32-C6-LCD-1.47/ESP32-C6-LCD-1.47_schemetics.pdf
[Resultados e histórico](hardware-validation.md).
