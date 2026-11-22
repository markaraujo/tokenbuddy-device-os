#include "control.h"
#include <string.h>
#include <stdint.h>
#define CHECK(c) do {if (!(c)) return false;} while (0)
bool control_tests(void)
{
    tb_model m;tb_behavior b;tb_model_init(&m);tb_init(&b,0);
    tb_reply r=tb_command(&m,&b,"{\"v\":1,\"id\":1,\"type\":\"heartbeat\"}",100);
    CHECK(!r.error && r.id==1 && m.connected);
    CHECK(!m.telemetry_known);
    r=tb_command(&m,&b,"{\"v\":1,\"id\":2,\"type\":\"settings\",\"brightness\":40,\"animation\":99}",110);
    CHECK(r.error && m.brightness==30 && m.animation==1);
    CHECK(tb_command(&m,&b,"{\"v\":1,\"id\":3,\"type\":\"navigate\",\"page\":5}",120).error);
    CHECK(m.page==0);
    CHECK(!tb_command(&m,&b,"{\"v\":1,\"id\":4,\"type\":\"event\",\"event\":\"agent.started\",\"agent\":\"Codex\"}",200).error);
    CHECK(m.agent_running && tb_sample(&b,210).state==TB_WORKING && !m.tokens_known);
    CHECK(!tb_command(&m,&b,"{\"v\":1,\"id\":5,\"type\":\"event\",\"event\":\"agent.completed\",\"agent\":\"Codex\",\"input_tokens\":123,\"output_tokens\":45}",300).error);
    CHECK(!m.agent_running && m.tokens_known && m.input_tokens==123 && tb_sample(&b,310).state==TB_SUCCESS);
    CHECK(tb_sample(&b,5300).state==TB_IDLE);
    tb_model_tick(&m,&b,5301);CHECK(!m.connected && b.base==TB_DISCONNECTED);
    CHECK(tb_command(&m,&b,"{\"v\":2,\"id\":6,\"type\":\"heartbeat\"}",0).error);
    CHECK(tb_command(&m,&b,"{\"v\":1,\"id\":7,\"type\":\"heartbeat\"} junk",0).error);
    CHECK(tb_command(&m,&b,"[[[[[[[[[0]]]]]]]]]",0).error);
    CHECK(tb_command(&m,&b,"{\"v\":1,\"id\":8,\"type\":\"telemetry\",\"cpu\":1e999}",0).error);
    for (unsigned i=0;i<20;++i) tb_add_event(&m,"bounded event");
    CHECK(m.event_count==TB_EVENTS && m.event_head<TB_EVENTS);
    tb_button button={0};
    CHECK(tb_button_poll(&button,true,0)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,10)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,50)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,true,100)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,true,130)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,200)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,230)==TB_BUTTON_SHORT);
    CHECK(tb_button_poll(&button,false,300)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,true,400)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,true,430)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,1230)==TB_BUTTON_NONE);
    CHECK(tb_button_poll(&button,false,1260)==TB_BUTTON_LONG);
    m.page=4;m.brightness=47;tb_button_action(&m,&b,TB_BUTTON_LONG,1290);CHECK(m.brightness==50);
    tb_button_action(&m,&b,TB_BUTTON_LONG,1300);CHECK(m.brightness==0);
    tb_button_action(&m,&b,TB_BUTTON_SHORT,1400);CHECK(m.page==0);
    tb_set_base(&b,TB_WORKING);tb_request(&b,TB_ERROR,2000,100);
    CHECK(tb_sample(&b,2100).state==TB_WORKING);
    return true;
}
