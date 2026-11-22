# Próximos critérios de aceite

1. Ambiente e build mínimo passaram; revisar diferença local do SDK e validar a seleção nativa da extensão VS Code antes de homologar a Fase 0.
2. COM3 identificada como ESP32-C6FH8/8 MB; confirmar revisão física da Waveshare diante da divergência de capacidade. Obter confirmação explícita para primeiro flash, verificar boot, heartbeat e reset.
3. Demo oficial baixado: LVGL 8.3.11 e ST7789T; validar build do exemplo no SDK candidato antes de integrar LCD, orientação, cores e backlight.
4. Avatar MVP e máquina de estados independente do hardware, com testes de transições e prioridades no host.

Nenhum flash, LCD ou teste físico de firmware próprio foi aprovado. Sem sensores ou software host antes de validar a base.

## Avanço da Fase 2

Demo Waveshare original e teste LCD isolado compilaram no SDK candidato.
[Teste e critérios](lcd-validation.md): fonte versionada, brilho inicial 30%,
sem montagem/formatação microSD. Teste físico aguarda nome/revisão da placa,
pois a unidade FH8/8 MB diverge do esquema sem touch FH4/4 MB.
Nenhuma nova gravação foi executada nesta etapa.

Teste físico LCD concluído inicialmente: foto do verso confirmou o modelo sem
touch; cores/borda/orientação aprovadas pelo usuário, com logs dos cinco padrões.
A pendência de identificação física citada acima foi resolvida. Estabilidade
prolongada e homologação do SDK limpo continuam pendentes. Próximo: avatar MVP.

## Avatar MVP implementado

Versão 0.2.0 gravada: olhos ciano, piscadas, olhar, feliz/concentrado/erro e RGB,
com demo local identificada. Testes C no host PASS; 60 s de logs sem crash observado,
heap do sistema constante. Aceite visual e teste prolongado pendentes.
Próximo ajuste deve partir da observação visual; não adicionar sensores ou host
antes desse aceite. SDK limpo continua pendente de homologação.
