# Host Windows e integração com agentes

## Uso normal

```powershell
./tools/host.ps1 setup   # ambiente isolado e dependências fixadas
./tools/host.ps1 start   # processo local oculto; sem instalar serviço global
./tools/tokenbuddy.ps1 status
./tools/tokenbuddy.ps1 navigate 2
./tools/tokenbuddy.ps1 settings --brightness 30 --animation 1 --demo off
./tools/tokenbuddy.ps1 expression happy --duration 3000
./tools/host.ps1 stop
```

Configuração local: `.tools/host-config.json`, fora do Git. Porta COM3, MAC
14:c1:9f:05:7b:ec e HTTP 8765 para a unidade validada. Uma chave aleatória é
criada uma única vez; não é impressa, publicada nem enviada ao ESP32. A API escuta
apenas 127.0.0.1 e exige X-TokenBuddy-Key em todas as rotas /api. Não há CORS aberto.
Health: http://127.0.0.1:8765/health. Schema de API: http://127.0.0.1:8765/docs.
O CLI lê a chave local automaticamente. Nenhuma configuração global, autostart,
driver ou porta de firewall é alterada pelo host.

API: GET /api/status; POST /api/event, /api/settings, /api/navigate, /api/expression.
Validação estrita rejeita números fora dos limites, campos extras e tokens
incompletos. Ack confirma aplicação do comando no firmware; não é apenas enqueue.
UI pode receber atualização no status periódico seguinte.

CPU é uma amostra real de 100 ms via psutil; RAM é virtual_memory. NVIDIA é lida
via nvidia-smi (primeira GPU, timeout 1,5 s), sem interpretar a UI de programas.
GPU/VRAM ausentes são omitidas, não substituídas por zero. Heartbeat a cada segundo,
métricas aproximadamente a cada 2 s. Firmware sinaliza offline após 5 s sem host.
Status host distingue porta conectada de dados recentes e mantém os últimos 16 erros.

## Eventos de processos reais

```powershell
./tools/tokenbuddy.ps1 run --agent Build -- powershell -c "Write-Output 'ok'"
```

O wrapper acompanha o processo solicitado: started antes do início, completed com
exit 0 e failed com saída diferente de zero. Não extrai nem estima tokens. Preserva
a saída do processo e o código de saída. Se o host estiver indisponível, avisa no
stderr e não impede a ferramenta de desenvolvimento de executar.

## Codex CLI

```powershell
./tools/tokenbuddy.ps1 codex -- "Resuma a estrutura deste repositorio sem alterar arquivos"
```

Executa `codex exec --json` com os argumentos do usuário e a autenticação normal
da CLI. Não muda modelo, trust, sandbox, permissões ou arquivos globais do Codex.
A execução de um modelo só acontece quando esse comando é invocado pelo usuário;
não foi iniciada uma chamada adicional de LLM para testar o conector.

Mapeamento oficial: turn.started -> WORKING; turn.completed -> SUCCESS;
turn.failed (ou término de processo com falha) -> ERROR. input_tokens e
output_tokens vêm de usage do turno concluído. cached_input_tokens não é somado
novamente à entrada. Não representam contexto, cota, créditos ou custo.
Eventos item.* e diagnósticos error isolados não inventam conclusão de turno.
Uma nova tarefa cria task_id; o MVP rejeita concorrência de outra tarefa ativa.
Não há monitoramento automático de sessões Codex já abertas ou parsing do terminal.

Contrato conferido na CLI instalada 0.171.0 (`codex exec --help`) e documentação:
https://developers.openai.com/codex/noninteractive/
Conector validado com eventos JSONL de teste; o caminho completo de processos reais
foi validado usando subprocessos locais de sucesso/falha, sem consumir uma chamada LLM.

## Testes

```powershell
./tools/test.ps1
python tools/test-hardware.py --soak-seconds 180
./tools/host.ps1 stop
python tools/test-recovery.py
./tools/host.ps1 start
```

Os testes de hardware mudam telas, brilho e estados reais. test-recovery exige
host parado, lê a serial diretamente e reinicia a placa via esptool, sem gravar.
BOOT físico é tratado pelo firmware; RESET mantém a função original.
Logs/relatórios e backup da flash permanecem em `.tools`, fora do Git.
