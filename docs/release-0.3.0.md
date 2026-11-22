# Entrega 0.3.0 — MVP integrado

## O que está funcionando

Firmware para a unidade ESP32-C6-LCD-1.47 sem touch fotografada e identificada:
8 MB, COM3, MAC-48 14:c1:9f:05:7b:ec. SDK ESP-IDF 5.5.2 limpo; driver LCD oficial
validado fisicamente; LVGL 8.3.11 e led_strip 2.5.5 fixados em lock.

Dez estados, olhos ciano, piscadas, olhar, RGB, cinco telas, BOOT com debounce/
acionamento longo, ajustes persistentes e funcionamento offline. RESET não foi
reaproveitado. Demo local pode ser ligada no avatar por BOOT longo, sem fingir
eventos reais de agentes. APIs/CLI também permitem navegar e ajustar animação.

Host Windows local ativo, interface apenas 127.0.0.1:8765, autenticação em /api,
identificação USB antes de comandos, heartbeat, CPU/RAM reais e NVIDIA opcional.
Conector de processos envia início/sucesso/falha reais; Codex usa o contrato JSONL
oficial. Nenhum token, custo, contexto ou cota é estimado. Sem chamada LLM extra
para validar o conector; o caminho Codex foi testado por eventos estruturados de
referência, e o caminho de execução foi testado com subprocessos locais reais.

## Evidência final

| Verificação | Resultado |
|---|---|
| Build final / flash | Exit 0, hashes verificados |
| Binário | 497920 bytes |
| Testes C de behavior/control | PASS no Windows e no boot |
| Testes Python | 8 testes PASS |
| Dependências Python | pip check sem conflitos |
| Cinco telas e dez estados | ACK/status reais no dispositivo |
| Processos reais exit 0 / exit 7 | SUCCESS / ERROR no firmware |
| Recuperação USB | Offline, linha longa, JSON inválido e limites: PASS |
| Ajustes após reset | Brilho/animação preservados por NVS |
| Observação final | 180,4 s, 90 amostras, sem reset |
| Heap do sistema | 374464 bytes em todas as amostras |
| LVGL livre | 26180–26232 bytes, fragmentação máxima 1% |
| Margem mínima da stack principal | 3640 bytes |
| Scripts PowerShell / JSON VS Code | Sintaxe validada |

SHA256 da imagem final:
`8AF2AC15D8670ACD4EB5F62B89CABD04289837A41C3171ED3558C38DAAD2F965`.

Testes reais em `.tools/integrated-hardware-report.json` e
`.tools/recovery-hardware-report.json`. O pacote inclui cópias desses relatórios
sem a chave local ou backup original. Backup original de 8388608 bytes permanece
local: SHA256 `0463BF6DE6FFBACB9A9F3F54B0A6923A689990905E9EC91D4EC9E86A0F84462D`.

O usuário aceitou visualmente LCD e avatar 0.2.0. As novas telas/estados 0.3.0
foram exercitadas por protocolo e não fotografadas para comparação visual final.
Contadores GPIO registraram 15 acionamentos curtos e um longo na etapa anterior;
a sequência de botão também tem testes unitários determinísticos.

## Incidentes corrigidos e limites

- MAC: API eFuse C6 retorna EUI-64 de 8 bytes; buffer de seis era incorreto.
  Identificação recusou o endereço truncado. Corrigido para esp_read_mac STA
  MAC-48, sem ligar Wi-Fi; identidade física reconfirmada.
- PowerShell: stderr informativo de SDK/pip interpretado como erro sob redireção.
  Wrappers passaram a verificar exit code nativo, preservando falhas reais.
- Brilho via BOOT: 47% poderia chegar a 57%; agora avança ao próximo nível até 50%.
- Telemetria inicial: antes do primeiro pacote, a UI mostra aguardando dados,
  em vez de zeros fictícios. Dados expiram ao perder host.
- Primeiro teste longo teve interrupção USB perto de 161 s. Causa não determinada;
  diagnóstico preservado, sem atribuir ao usuário ou declarar causa corrigida.
  Reset por esptool restaurou o canal. Acrescentados motivo de reset/histórico
  de erros; observações posteriores, incluindo a imagem final, passaram 180 s.
  Esses testes não provam estabilidade indefinida nem ausência de todos os leaks.

## Pacote e operação

Pacote local gerado por tools/package-release.ps1, com fontes versionadas,
firmware, manifest/hashes e relatórios. SDK/toolchain, dependências Python,
chave de API e backup da flash não são distribuídos dentro do ZIP.
[Uso do host](host.md) e [setup](setup-windows.md) contêm os comandos reproduzíveis.
Host está configurado para a unidade validada; não inicia com o Windows nem muda
drivers/firewall. Parar com tools/host.ps1 stop antes de monitor/flash externo.

## O que não faz parte desta conclusão

Não foram inventados sensores ou testadas peças ausentes. IMU externa/MPR121,
Wi-Fi/provisionamento, conversação/voz/LLMs locais/nuvem/memória, dashboard Next.js,
Claude/OpenCode e OTA permanecem no roadmap. O MVP usa USB e acompanha uma tarefa
por vez. O desenvolvimento de todo o produto futuro não é declarado concluído.

## Refinamento visual solicitado depois do aceite funcional

Após os testes acima, o usuário solicitou uma interface mais elaborada e dispensou
nova validação. Renderer foi refinado com glow em camadas, crossfade de expressões,
piscadas suavizadas, olhar interpolado, respiração leve e menor região de redesenho.
Telas secundárias passaram a usar cards reutilizáveis, bordas neon discretas,
margem externa uniforme, tipografia hierárquica, indicadores e barras animadas de
métricas reais. Tokens ausentes seguem como --, sem percentuais fictícios.

Imagem funcional testada foi preservada em .tools/validated-base-0.3.0. A revisão
visual final tem build e gravação verificados, sem repetir o teste longo nem pedir
aceite visual, conforme solicitado. O SHA/size e as medições na tabela anterior
referem-se à baseline funcional. Manifest do pacote distingue a imagem visual
atual e a baseline; não apresenta a revisão como submetida à mesma validação longa.
FPS não foi medido após esse refinamento; qualidade visual não é declarada como
certificada por inspeção automatizada do LCD.

Imagem da revisão visual: 585376 bytes, SHA256
`2F305FF57874D6CA856B6CFE70560964222A53266D5FD621783258A3AF6CB7A7`.
Build e flash terminaram com exit 0; o host reconectou à MAC correta e recebeu
status/telemetria. Borda global de 1 px com cantos arredondados, cards com margem
lateral de 10 px, fontes Montserrat 10/12/16/18/22, gradientes escuros e acentos
ciano/verde/violeta. Barras mostram uso real; o feixe de agente é atividade sem
percentual de progresso inventado. Protocolo e lógica funcional permaneceram iguais.
