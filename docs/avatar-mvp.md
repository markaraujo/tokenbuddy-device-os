# Avatar MVP — TokenBuddy Device OS 0.2.0

## Implementação

- `board`: SPI/LCD ST7789T, backlight PWM limitado a 50% (uso inicial 30%),
  RGB integrado GPIO8 com níveis baixos, microSD mantido deselecionado.
- `display`: LVGL 8.3.11, dois buffers estáticos de 172 x 20 pixels, tick 2 ms,
  conclusão de transferência SPI libera o buffer; uma única tarefa manipula a UI.
- `behavior`: C puro, estados e prioridades, expiração, piscadas e olhar determinísticos;
  não inclui ESP-IDF, FreeRTOS ou LVGL.
- `avatar`: desenho LVGL sem criar objetos por frame; olhos ciano, arco feliz,
  olhos concentrados, X vermelho de erro e piscadas. Atualização aproximadamente 30 ms.
- `main`: modo de demonstração local, ciclo repouso/feliz/concentrado/erro a cada 6 s;
  a faixa DEMO LOCAL distingue a demonstração de eventos reais de agentes.

Hardware: mesma unidade sem touch validada na COM3. LCD: MOSI6/SCLK7/CS14/DC15/
RST21/BL22, offset34/0, SPI12 MHz. MicroSD CS4 deselecionado; nada é montado ou formatado.
RGB GPIO8 conferido no esquema sem touch e exemplo oficial. Sem entradas touch,
IMU, rede, áudio, credenciais ou métricas inventadas. Flash configurada para 8 MB,
conforme chip/flash_id; partição de aplicação permanece 1 MB.

## Dependências e procedência

LVGL 8.3.11 e led_strip 2.5.5 fixados em manifests e dependencies.lock.
Driver ST7789T Waveshare preservado em board/vendor, com autoria/licença e origem.
ESP-IDF instalado continua v5.5.2-dirty com a diferença previamente documentada;
essa limitação de homologação não foi ocultada nem corrigida automaticamente.

## Testes de comportamento no Windows

Nenhum compilador C nativo foi encontrado no PATH ou na consulta VS Build Tools.
TinyCC portátil 0.9.27 foi obtido localmente a partir do servidor de releases
referenciado pelo site do projeto, sem alterar PATH ou instalação global:
https://bellard.org/tcc/
https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27-win64-bin.zip

SHA256 ZIP: 34A721949A2583FDFF725312DA092FA0F5F1F284B702E6F811C6954714FAABB2.
Extração em `.tools/tinycc` (executável `.tools/tinycc/tcc/tcc.exe`).

```powershell
./tools/test-behavior.ps1
. ./tools/activate.ps1
./tools/idf.ps1 build
```

Os testes C executados no host verificam prioridade do erro, bloqueio de eventos
inferiores, expiração na fronteira, substituição por prioridade superior, pedidos
inválidos, contador uint32_t atravessando o máximo, fechamento/abertura de piscada
e limites do olhar e abertura. Resultado: PASS.
O mesmo teste é executado no boot desta versão de desenvolvimento; falha aborta
antes de inicializar hardware. O código de teste também aceita compilador C99
convencional: definir TOKENBUDDY_HOST_TEST, incluir behavior/include e compilar
behavior.c junto de tests/test_behavior.c.

Build incremental concluído com exit 0. Binário: 440064 bytes; configuração flash
8 MB. Gravação COM3 autorizada no contexto do projeto, identidade reconfirmada por
MAC base 14:c1:9f:05:7b:ec; flash terminou com exit 0 e hashes verificados.
Backup anterior integral preservado em .tools/device-before-tokenbuddy-8mb.bin.
Logs: .tools/build-avatar.log, .tools/flash-avatar.log e .tools/avatar-serial.log.

## Limites de aceitação

A captura de logs verifica execução e heap do sistema; não comprova ausência de
vazamentos no heap interno do LVGL nem substitui teste prolongado. Fluidez,
expressões e RGB precisam da observação do usuário. Sem FPS medido ou alegação de
estabilidade de longa duração. Intensidade de animação e eventos reais ficam para
iterações posteriores ao aceite visual.

## Execução observada

Captura real de 60 segundos concluída com PASS: todos os quatro estados presentes,
12 amostras de heap do sistema iguais a 396888 bytes e selftest=PASS; sem panic ou
abort nos logs observados. Sem alegação de FPS medido ou teste de longa duração.
Aceite visual do avatar e LED solicitado ao usuário, ainda pendente nesta entrega.
SHA256 do binário gravado:
F5911984F93F63CDFD5CA3BA6CDA62951AF2578EC66EBED0F6FE16AE555E0179.
