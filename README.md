# TokenBuddy AI — MVP 0.3.0

Companheiro físico para Waveshare ESP32-C6-LCD-1.47 **sem touch**, unidade FH8/8 MB.
Firmware gravado e host Windows funcionando por USB na COM3.

- Avatar ciano com piscadas, olhar e dez expressões/estados.
- Telas Avatar, Agentes, Sistema, Eventos e Ajustes.
- BOOT curto navega; longo no avatar alterna demo, em Ajustes muda brilho,
  nas outras telas volta ao avatar. RESET mantém a função original.
- CPU/RAM e GPU/VRAM NVIDIA reais, sem métricas inventadas quando indisponíveis.
- Eventos de processos e conector de `codex exec --json`; uma tarefa ativa por vez.
- Ajustes persistidos, heartbeat/offline, API local autenticada e diagnóstico.

```powershell
./tools/host.ps1 start
./tools/tokenbuddy.ps1 status
./tools/tokenbuddy.ps1 navigate 2
./tools/tokenbuddy.ps1 codex -- "Resuma este repositorio sem alterar arquivos"
```

O comando codex acima executa o modelo quando invocado pelo usuário; não foi usado
para consumir uma chamada LLM adicional nos testes. Não monitora automaticamente
sessões já abertas. Tokens vêm apenas de usage dos eventos do Codex.

## Desenvolvimento

```powershell
./tools/host.ps1 setup
./tools/test.ps1
. ./tools/activate.ps1
./tools/idf.ps1 build
./tools/idf.ps1 size
```

VS Code: tarefa padrão de build, tarefas de testes/start/stop host e perfil de
terminal TokenBuddy ESP-IDF. Nenhuma configuração global do editor foi alterada.
SDK limpo v5.5.2, LVGL 8.3.11, led_strip 2.5.5; dependências fixadas.

Flash deliberado, após conferir placa/porta e parar o host:

```powershell
./tools/host.ps1 stop
./tools/flash.ps1 -Port COM3 -ExpectedMac 14:c1:9f:05:7b:ec -Confirmed
./tools/host.ps1 start
```

Backup completo do firmware original preservado em `.tools`, fora do Git e do
pacote de entrega. A versão exige a unidade sem touch de 8 MB validada; não usar
na variante Touch/JD9853 ou presumir compatibilidade automática com placa 4 MB.

## Validação e escopo

Testes C no Windows e no boot, oito testes Python, troca real API/USB/firmware,
recuperação de mensagens inválidas, offline, persistência após reset e teste final
de 180 segundos passaram. Não equivale a certificação ou estabilidade de longa duração.

O MVP disponível hoje está entregue. IMU/MPR121 externos, Wi-Fi/provisionamento,
voz/IA conversacional/memória, dashboard Next.js, outros conectores e OTA continuam
etapas futuras, não simuladas nem declaradas concluídas. Esta placa não possui
acelerômetro integrado segundo o esquema sem touch.

- [Entrega e evidências](docs/release-0.3.0.md)
- [Host e Codex](docs/host.md)
- [Protocolo USB](docs/protocol.md)
- [Hardware](docs/hardware.md)
- [Setup Windows](docs/setup-windows.md)
- [Arquitetura](docs/architecture.md)
- [Etapas concluídas e futuras](docs/development-plan.md)
- [Referência visual](docs/ui-reference.md)

Refinamento visual posterior: cards neon, borda externa fina, margens consistentes,
fontes hierárquicas e efeitos suaves do avatar. Build/gravação da revisão visual
verificados; teste longo é da baseline funcional, sem repetição a pedido do usuário.
O pacote identifica essa distinção em manifest e validation/visual-revision.txt.
