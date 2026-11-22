# Validação inicial do hardware — 2026-11-22

## Resultados observados, sem gravação

| Verificação | Evidência |
|---|---|
| USB | COM3, VID 303A/PID 1001; consultas esptool concluídas |
| MCU | ESP32-C6FH8 QFN32, revisão v0.2, cristal 40 MHz |
| Identidade | MAC base 14:c1:9f:05:7b:ec |
| Flash | 8 MB, fabricante 0x20, device 0x4017; flash_id confirmado |
| Segurança | Secure Boot desativado; Flash Encryption desativada |
| Leitura persistente | Tabela em 0x8000 e cabeçalho em 0x10000 lidos com sucesso |
| Firmware existente | Descritor válido, projeto arduino-lib-builder, versão 487f743, IDF v5.5.1-255-g07e9bf4970 |
| Execução | 495 bytes de logs capturados por 8 segundos a 115200 baud |
| Bluetooth | Firmware existente reportou 8 dispositivos encontrados |
| RAM | Firmware reportou SRAM livre 227896 bytes; sem teste independente de integridade |
| microSD | Firmware reportou No SD card attached e falha de inicialização; presença física não confirmada |
| Wi-Fi | Mensagem de início do teste vista; resultado não capturado |
| LCD/RGB | Não validados visualmente |

A tabela instalada ocupa os primeiros 4 MB: nvs (0x9000), otadata (0xe000),
app0 (0x10000, 3 MB), spiffs (0x310000, 896 KB), coredump (0x3f0000, 64 KB).
Isso não reduz a capacidade física de 8 MB e não identifica o modelo da placa.
Nenhum conteúdo NVS ou credencial foi lido. Apenas tabela e cabeçalho foram extraídos.
Consultas esptool reiniciaram o dispositivo e algumas executaram stub em RAM;
nenhum firmware persistente ou eFuse foi gravado.

## Divergência de identificação

O esquema oficial sem touch declara ESP32-C6FH4; a unidade conectada contém FH8.
Não concluir que é touch nem aplicar pinagem alternativa sem evidência física.
Solicitar fotos nítidas da frente e do verso, inscrições/revisão e indicação do que
aparece atualmente no LCD. Pode haver revisão de hardware ou outra variante;
causa ainda não determinada. Não inicializar GPIOs de LCD/RGB com base em suposição.

Esquema oficial baixado em .tools/waveshare-schematic.pdf.
SHA256: D8DED2DA08268E27148C700B38C2B1BA62BCA2B734153C7C6E88ECC5E5C72E51.
Fonte: https://files.waveshare.com/wiki/ESP32-C6-LCD-1.47/ESP32-C6-LCD-1.47_schemetics.pdf

Logs brutos locais em .tools/hardware-security.log, .tools/flash-id.log,
.tools/device-inspection.log, .tools/device-passive-serial.log e
.tools/device-partitions.csv. Mantidos fora do Git.

Próximo critério: correlacionar identificação física e esquema, confirmar LCD/LED
visualmente e preparar teste específico. Primeiro flash ainda requer confirmação
explícita para a COM3 e o firmware escolhido.

## Primeiro flash TokenBuddy e execução real

Após autorização explícita para a COM3, identidade MAC/chip reconfirmada.
Backup completo anterior: .tools/device-before-tokenbuddy-8mb.bin, 8388608 bytes;
esptool read_flash terminou com exit 0. SHA256 em .tools/backup-flash.sha256.txt.
Backup mantido local, fora do Git; pode conter dados do firmware anterior.

`idf.py -C firmware/esp32-c6 -p COM3 flash` terminou com exit 0.
Bootloader (22592 bytes), aplicação (147456 bytes) e tabela (3072 bytes)
gravados e hashes verificados pelo esptool. Sem erase_flash completo.
Imagem mínima conserva a configuração de 4 MB; hardware físico continua sendo 8 MB.
LCD e LED não inicializados por este firmware.

Primeira captura após reset perdeu a interface USB (ClearCommError). Captura
seguinte ficou vazia; usuário informou acionamento acidental de botão, trazendo
nova evidência para possível alteração do estado de boot. Console USB confirmado
no sdkconfig; nenhuma alteração de código como tentativa de correção.
Executado esptool run com hard_reset, exit 0. Reaberta COM3, capturados quatro
heartbeats TokenBuddy nos tempos 5090/10090/15090/20090 ms, heap livre constante
464108 bytes. Nenhum crash observado nessa janela de aproximadamente 20 segundos.
Isso valida execução e atividade periódica; não substitui teste longo de estabilidade.
Logs: .tools/first-flash.log, .tools/recover-button-reset.log,
.tools/tokenbuddy-first-boot.log. Banner inicial não capturado; execução confirmada
pelas mensagens exclusivas do firmware mínimo. LCD com firmware próprio pendente.

Referência visual do usuário localizada em Downloads e preservada no repositório:
[Diretrizes da UI](ui-reference.md). Próximo marco é validar driver/pinagem LCD
antes de integrar o avatar ciano e animações da referência.
