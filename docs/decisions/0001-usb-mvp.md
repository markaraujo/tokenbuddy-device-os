# ADR 0001 — USB para o primeiro MVP integrado

Status: adotado.

A unidade já usa USB-C para energia, flash e console. Não há credenciais Wi-Fi
fornecidas e a base de LCD foi validada nesse transporte. Adotamos JSON por linha
sobre USB Serial/JTAG para ligar firmware e host local FastAPI.

Isso entrega eventos e métricas reais sem provisionamento de rede ou credenciais
no firmware. O contrato lógico v1 pode ser transportado por Wi-Fi depois; nenhum
módulo Wi-Fi vazio ou protocolo MQTT sem necessidade foi adicionado.

BOOT GPIO9, conferido no esquema sem touch, permite navegação provisória com
debounce e pressionamento longo. RESET continua físico; MPR121/IMU só serão
integrados quando os módulos estiverem disponíveis. Não usar BOOT mantido durante
reset para navegar: isso pode entrar no bootloader ROM, conforme projeto da placa.

Limites: USB precisa do computador; host local acompanha uma tarefa ativa por vez.
Não substitui o plano futuro de rede, sensores, voz, memória contextual e OTA.
