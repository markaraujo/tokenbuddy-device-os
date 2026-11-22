# Referência de interface TokenBuddy

Referência fornecida pelo usuário: [interface neon](reference/tokenbuddy-ui-neon.png),
localizada em Downloads em 2026-11-22. É direção visual, não especificação de hardware.

- Canvas do dispositivo: 172 x 320, orientação vertical.
- Fundo preto/azul muito escuro; destaque ciano #00E5FF; olhos grandes com brilho moderado.
- Estados: repouso, feliz, curioso, pensando, trabalhando, sucesso, erro, tonto, dormindo e desconectado.
- Fluxo futuro: avatar, agentes, sistema, eventos, ajustes. Só criar telas quando houver função real.
- MVP visual: avatar central, piscadas, movimento dos olhos, expressões e saudação curta; sem métricas fictícias.
- Cabeçalho e rodapé discretos; texto legível em tamanho real, sem reduzir o painel conceitual inteiro para o LCD.
- Navegação física depende dos sensores externos ainda não validados; preparar entrada simulável quando necessário.

Correções e limites da referência:
- ESP32-C6: Wi-Fi 2,4 GHz; não oferecer rede 5 GHz.
- Percentuais de contexto, uso de tokens, cotas e custos são métricas diferentes; exibir somente dados disponíveis.
- Brilho inicial deve respeitar o limite conservador de 50% da documentação, não o exemplo de 70% da arte.
- Sem áudio, IMU, sensores ou LEDs externos presumidos instalados.
- Não adicionar credenciais nem redes ilustrativas da referência ao firmware.

Primeiro flash permanece mínimo, com console USB e sem inicialização de LCD/GPIOs.
Integração da UI exige confirmar o hardware de display antes de ativar sua pinagem.
