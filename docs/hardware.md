# Hardware

Alvo: Waveshare ESP32-C6-LCD-1.47, sem touch; ESP32-C6, flash 4 MB, ST7789, 172 × 320.
Pinagem confirmada na documentação oficial web:

| Sinal | GPIO |
|---|---|
| LCD MOSI | 6 |
| LCD SCLK | 7 |
| LCD CS | 14 |
| LCD DC | 15 |
| LCD RESET | 21 |
| LCD BL | 22 |
| RGB | 8 |

microSD compartilha MOSI/SCLK; CS=4, MISO=5. Não alocar outros pinos sem revisar esquema.
Fabricante recomenda brilho de no máximo 50%. Implementar PWM antes de habilitar backlight.
Esquema e código demo ainda não foram baixados/revisados; offsets, orientação, sequência de inicialização e ordem de cores permanecem pendentes.
MPR121, IMU, áudio, bateria e LEDs externos não presumidos presentes.

Fonte consultada: https://docs.waveshare.com/ESP32-C6-LCD-1.47

Verificação real em 2026-11-22: COM3 contém ESP32-C6FH8 com 8 MB, confirmado por
esptool chip_id e flash_id. A identificação física da revisão Waveshare permanece
pendente. O demo oficial foi baixado e revisado parcialmente: LVGL 8.3.11,
ST7789T próprio, offset X=34/Y=0, SPI 12 MHz, espelhamento horizontal.
Não houve teste LCD ou gravação. Informações anteriores de demo não baixado foram
superadas por esta verificação; revisão do esquemático permanece pendente.
