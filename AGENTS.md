# TokenBuddy AI

- ESP-IDF/FreeRTOS para Waveshare ESP32-C6-LCD-1.47 sem touch, unidade FH8/8 MB validada. Nao confundir com Touch/JD9853.
- SDK limpo v5.5.2; LVGL 8.3.11 e led_strip 2.5.5 fixados. Nao atualizar sem verificar compatibilidade.
- Ambiente: `. ./tools/activate.ps1`; build: `./tools/idf.ps1 build`; testes: `./tools/test.ps1`. IDF_PATH/IDF_TOOLS_PATH locais; export.ps1 do SDK.
- board/display isolam hardware; behavior/control sao C testavel no host; avatar/ui desenham; connectivity faz USB; main coordena. Host em services/core.
- Host: `./tools/host.ps1 setup|start|stop|status`; CLI: `./tools/tokenbuddy.ps1 ...`. Chave em .tools fora do Git; nao enviar a chave ao ESP32.
- BOOT GPIO9 e entrada com debounce; RESET permanece fisico. Nao alocar GPIOs sem esquema/conflitos microSD/USB.
- Nunca gravar sem identificar chip/porta e confirmacao explicita. tools/flash.ps1 exige porta, MAC e -Confirmed. Parar host antes do flash; unidade sem touch/8 MB.
- Nao apagar arquivos, alterar drivers ou configuracao global sem autorizacao. Backup original em .tools nao entra no pacote/Git.
- Registrar causa provavel antes de corrigir; maximo duas tentativas sem evidencia nova. Nao usar clean como solucao generica nem refatorar modulos alheios.
- Nao declarar build, flash, teste fisico, validacao visual ou tokens reais sem resultado. Distinguir fixture, processo real e chamada de modelo.
- Se uma revisao visual nao repetir os testes por instrucao do usuario, registrar isso no manifest; preservar imagem/relatorios da baseline validada.
- Nao adicionar modulos futuros vazios. IMU/MPR121, Wi-Fi, voz, memoria/LLMs, dashboard e OTA exigem recursos e requisitos especificos.
