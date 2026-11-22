# Revisão de legibilidade — 0.3.0 / legibilidade

Solicitação: textos pequenos e aparentando pouca nitidez/opacidade no LCD.
Causa provável identificada no tema: legendas de 10 px, cores secundárias escuras
e fade aplicado à tela inteira. Resolução física permanece 172 x 320.

Mudanças somente no renderer/layout:
- Legendas, conexão, estados e rodapé passaram para 12 px, com cores mais claras.
- Valores CPU/RAM/GPU passaram para 18 px; VRAM mantém 16 px para caber.
- Eventos usam 14 px e separam nome curto do agente e ação em duas linhas.
- Opacidade do texto explicitamente em 100%; removido fade da página inteira.
- Cards de métricas/eventos ganharam altura e posições ajustadas para fontes maiores.
- Mantidos olhos com glow, barras suaves, borda externa, margens e brilho físico 30%.

Build incremental e flash na unidade ESP32-C6 sem touch/8 MB terminaram com exit 0,
com identidade reconfirmada e hashes verificados. Não houve nova inspeção visual
ou repetição dos testes longos, conforme preferência do usuário. A base funcional
continua preservada em .tools/validated-base-0.3.0; pacotes distinguem essa evidência.

Imagem: 576048 bytes.
SHA256: 806B646F0F4F7546BB0E12BC1554A2BF4A79F1F683D077F352341454CF194C86.
Logs: .tools/build-readability.log e .tools/flash-readability.log.
Não foram alterados driver LCD, pinagem, protocolo, sensores ou comportamento do host.
