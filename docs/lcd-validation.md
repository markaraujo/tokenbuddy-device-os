# Validação do LCD — candidato sem touch

## Evidência de compilação

Demo oficial Waveshare sem alterações de fonte: build concluído (exit 0) no
ESP-IDF v5.5.2-dirty, com LVGL 8.3.11 e led_strip 2.5.5.
Log: `.tools/build-waveshare-original.log`.
Isso comprova compilação nessa instalação, não compatibilidade física com a
unidade conectada nem homologação do SDK oficial intacto.

Arquivo original obtido de:
https://files.waveshare.com/wiki/ESP32-C6-LCD-1.47/ESP32-C6-LCD-1.47-Demo.zip
SHA256: 4B960E8CCD58E54E60494A2A6DA62531695C9152C9950153CAD34ACC496C4EE4.
Fontes originais permanecem no arquivo ZIP. Nenhum flash do demo executado.

## Teste isolado

`tools/prepare-lcd-validation.ps1` cria `.tools/lcd-validation` a partir do demo
local. Preserva o exemplo original e não grava firmware. LVGL copiado é 8.3.11;
o driver ST7789T oficial conserva seus avisos de licença Apache-2.0.

```powershell
./tools/prepare-lcd-validation.ps1 # uma única vez; recusa sobrescrever projeto existente
. ./tools/activate.ps1
python "$env:IDF_PATH/tools/idf.py" -C .tools/lcd-validation build
```

O main usado está versionado em `tools/lcd-validation/main.c`.
Não depende da montagem do microSD para inicializar SPI. Não inicializa Wi-Fi,
Bluetooth, RGB, sensores ou serviços host. Mantém CS do microSD alto conforme
esquema oficial sem touch; não monta, escreve nem formata o cartão.
Brilho de 30% desde a primeira ativação, inclusive dentro de LCD_Init.

Sequência visual: vermelho, verde, azul, branco e preto, 3 segundos por padrão.
Borda de 2 pixels, resolução 172 x 320, texto ciano central e identificação TOP.
Logs registram padrão e heap livre. A aceitação exige observar cores corretas,
texto legível, borda inteira, orientação e ausência de resets/artefatos.
Não é teste prolongado nem medição de integridade de memória.

## Por que não gravar ainda

A unidade tem ESP32-C6FH8/8 MB. O esquema sem touch usa FH4/4 MB, LCD ST7789,
SPI GPIO6/7, reset21 e backlight22. A variante Touch oficial usa FH8/8 MB,
LCD JD9853 e GPIOs diferentes (SPI1/2, reset22, backlight23).
Apenas a capacidade e o chip não provam a variante. O usuário informou que
acha não haver touch, mas ainda não confirmou a inscrição da placa.
É necessário nome/revisão legível do verso ou foto nítida para correlacionar
com o esquema antes de ativar GPIOs. Não tentar pinagens por tentativa e erro.

Fontes oficiais comparadas:
- https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47
- https://files.waveshare.com/wiki/ESP32-C6-LCD-1.47/ESP32-C6-LCD-1.47_schemetics.pdf
- https://docs.waveshare.com/ESP32-C6-Touch-LCD-1.47

## Revisão do demo

O exemplo completo chama SD_Init, que contém format_if_mount_failed=true.
Também aplica BK_Light(75) dentro de LCD_Init antes de reduzir a 50% no main.
Por isso, o teste isolado remove a montagem SD e aplica 30% desde LCD_Init.
O driver usa SPI 12 MHz, RGB565/BGR, espelhamento horizontal e offset X=34/Y=0.

A alteração local do SDK em LOG_COMPRESS_SRCS segue preservada no diff e sem
origem determinada. Não é uma alteração de API LCD; o build observado passou,
mas não foi feita homologação como SDK limpo nem correção automática desse arquivo.

## Resultado do teste isolado

Build concluído com exit 0: `.tools/lcd-validation/build/tokenbuddy_lcd_validation.bin`,
420704 bytes. Log `.tools/build-lcd-validation.log`. Script de preparação passou
na análise de sintaxe PowerShell. Nenhum flash ou teste físico LCD executado.
Firmware mínimo TokenBuddy permanece na COM3.

## Identificação física e teste LCD concluídos

Foto fornecida em Downloads: `WhatsApp Image 2026-11-22 at 15.58.45.jpeg`.
Cópia versionada: [verso da placa](reference/board-back.jpeg).
Inscrição legível ESP32-C6-LCD-1.47, layout correspondente à placa sem touch.
Revisão de PCB não legível/identificada. Capacidade real permanece FH8/8 MB;
isso não altera o nome impresso nem autoriza presumir componentes touch.

Teste isolado ST7789/LVGL 8.3.11 gravado na COM3, exit 0, hashes verificados.
read_mac confirmou a mesma MAC base 14:c1:9f:05:7b:ec. Houve erro de binding de
parâmetros em Select-String no comando auxiliar; a comparação foi conferida no
log e refeita com -LiteralPath e -Pattern explícitos. Não houve divergência de MAC.

Captura serial de 25 segundos: oito mensagens, padrões 0/1/2/3/4 presentes,
heap livre constante de 400964 bytes; nenhum crash observado nessa janela.
Usuário confirmou explicitamente que cores, orientação, texto e borda estavam
corretos. Isso valida inicialmente a pinagem e inicialização ST7789 usadas
na unidade física. Não equivale a teste prolongado nem prova ausência de vazamento.
LCD operando com backlight 30%, SPI 12 MHz, offset 34/0, RGB565 e LVGL 8.3.11.
Logs locais: .tools/flash-lcd-validation.log e .tools/lcd-validation-serial.log.

Firmware atual: teste LCD, substituindo o mínimo sem display. Backup anterior
de 8 MB preservado em .tools/device-before-tokenbuddy-8mb.bin, fora do Git.
Próximo marco: integrar o driver validado em board/display e criar olhos ciano,
piscadas e expressões do avatar conforme a referência visual.
