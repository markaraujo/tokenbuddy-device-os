#include "control.h"
#include "cJSON.h"
#include <string.h>
#include <stdio.h>
#include <math.h>

void tb_add_event(tb_model *m, const char *text)
{
    snprintf(m->events[m->event_head],48,"%.47s",text);
    m->event_head=(m->event_head+1)%TB_EVENTS;
    if (m->event_count<TB_EVENTS) ++m->event_count;
}
void tb_model_init(tb_model *m)
{
    memset(m,0,sizeof(*m)); m->brightness=30; m->animation=1;
    strcpy(m->agent_status,"Sem tarefa"); tb_add_event(m,"TokenBuddy iniciado");
}
void tb_model_tick(tb_model *m, tb_behavior *b, uint32_t now)
{
    if (m->connected && (uint32_t)(now-m->last_seen)>5000) {
        m->connected=false; m->agent_running=false; m->telemetry_known=false;
        strcpy(m->agent_status,"Host desconectado");
        tb_add_event(m,"Host desconectado");
    }
    tb_set_base(b,m->connected?(m->agent_running?TB_WORKING:TB_IDLE):TB_DISCONNECTED);
}
tb_button_event tb_button_poll(tb_button *b, bool raw, uint32_t now)
{
    if (raw!=b->raw) {b->raw=raw; b->changed=now;}
    if (raw==b->stable || (uint32_t)(now-b->changed)<30) return TB_BUTTON_NONE;
    b->stable=raw;
    if (raw) {b->active=true; b->pressed=now; return TB_BUTTON_NONE;}
    if (!b->active) return TB_BUTTON_NONE;
    b->active=false;
    return (uint32_t)(now-b->pressed)>=800?TB_BUTTON_LONG:TB_BUTTON_SHORT;
}
void tb_button_action(tb_model *m, tb_behavior *b, tb_button_event event, uint32_t now)
{
    if (event==TB_BUTTON_SHORT) m->page=(m->page+1)%TB_PAGES;
    if (event==TB_BUTTON_LONG) {
        if (m->page==4) m->brightness=m->brightness>=50?0:(m->brightness/10+1)*10;
        else if (m->page==0) {m->demo=!m->demo; tb_add_event(m,m->demo?"Demo ativada":"Demo desativada");}
        else {m->page=0; tb_request(b,TB_HAPPY,now,3000);}
    }
}
static bool number(const cJSON *o,const char *name,double min,double max,double *value)
{
    const cJSON *v=cJSON_GetObjectItemCaseSensitive(o,name);
    if (!cJSON_IsNumber(v) || !isfinite(v->valuedouble) || v->valuedouble<min || v->valuedouble>max) return false;
    *value=v->valuedouble; return true;
}
static bool integer(const cJSON *o,const char *name,unsigned max,unsigned *value)
{
    double v;
    if (!number(o,name,0,max,&v) || v!=(unsigned)v) return false;
    *value=(unsigned)v; return true;
}
static const char *text(const cJSON *o,const char *name,size_t max)
{
    const cJSON *v=cJSON_GetObjectItemCaseSensitive(o,name);
    if (!cJSON_IsString(v) || strlen(v->valuestring)>max) return NULL;
    for (const unsigned char *p=(const unsigned char *)v->valuestring;*p;++p) if (*p<32 || *p>126) return NULL;
    return v->valuestring;
}
tb_reply tb_command(tb_model *model,tb_behavior *behavior,const char *line,uint32_t now)
{
    tb_reply reply={-1,"invalid_json"};
    if (strlen(line)>768) {reply.error="too_long"; return reply;}
    unsigned depth=0; bool quoted=false,escaped=false;
    for (const char *p=line;*p;++p) {
        if (quoted) {if (escaped) escaped=false; else if (*p=='\\') escaped=true; else if (*p=='"') quoted=false;}
        else if (*p=='"') quoted=true;
        else if (*p=='{' || *p=='[') {if (++depth>8) {reply.error="too_deep"; return reply;}}
        else if (*p=='}' || *p==']') {if (depth) --depth;}
    }
    cJSON *o=cJSON_ParseWithOpts(line,NULL,1);
    if (!o) return reply;
    tb_model m=*model; tb_behavior b=*behavior;
    unsigned id,version;
    if (!cJSON_IsObject(o) || !integer(o,"id",2147483647,&id)) goto done;
    reply.id=(int)id; reply.error="invalid_command";
    if (!integer(o,"v",1,&version) || version!=1) goto done;
    const char *type=text(o,"type",24);
    if (!type) goto done;
    if (!strcmp(type,"heartbeat")) {
        if (!m.connected) tb_add_event(&m,"Host conectado");
        m.connected=true; m.last_seen=now;
    } else if (!strcmp(type,"telemetry")) {
        if (!number(o,"cpu",0,100,&m.cpu) || !number(o,"ram",0,100,&m.ram) ||
            !number(o,"ram_used",0,1e9,&m.ram_used) || !number(o,"ram_total",1,1e9,&m.ram_total) || m.ram_used>m.ram_total) goto done;
        m.gpu_known=cJSON_HasObjectItem(o,"gpu");
        if (m.gpu_known && (!number(o,"gpu",0,100,&m.gpu) || !number(o,"vram_used",0,1e9,&m.vram_used) ||
            !number(o,"vram_total",1,1e9,&m.vram_total) || m.vram_used>m.vram_total)) goto done;
        m.telemetry_known=true;
        m.connected=true; m.last_seen=now;
    } else if (!strcmp(type,"navigate")) {
        unsigned page; if (!integer(o,"page",TB_PAGES-1,&page)) goto done;
        m.page=page;
    } else if (!strcmp(type,"settings")) {
        unsigned value;
        if (cJSON_HasObjectItem(o,"brightness")) {if (!integer(o,"brightness",50,&value)) goto done; m.brightness=value;}
        if (cJSON_HasObjectItem(o,"animation")) {if (!integer(o,"animation",2,&value)) goto done; m.animation=value;}
        if (cJSON_HasObjectItem(o,"demo")) {
            const cJSON *v=cJSON_GetObjectItemCaseSensitive(o,"demo");
            if (!cJSON_IsBool(v)) goto done;
            m.demo=cJSON_IsTrue(v);
        }
    } else if (!strcmp(type,"expression")) {
        unsigned state,duration;
        if (!integer(o,"state",TB_STATE_COUNT-1,&state) || !integer(o,"duration",60000,&duration) || duration<50) goto done;
        if (!tb_request(&b,(tb_state)state,now,duration)) {reply.error="priority"; goto done;}
    } else if (!strcmp(type,"event")) {
        const char *event=text(o,"event",32), *agent=text(o,"agent",31);
        if (!event || !agent || !*agent) goto done;
        bool start=!strcmp(event,"agent.started"),success=!strcmp(event,"agent.completed"),fail=!strcmp(event,"agent.failed");
        if (!start && !success && !fail) goto done;
        bool known=cJSON_HasObjectItem(o,"input_tokens") || cJSON_HasObjectItem(o,"output_tokens");
        unsigned in=0,out=0;
        if (known && (!integer(o,"input_tokens",2147483647,&in) || !integer(o,"output_tokens",2147483647,&out))) goto done;
        snprintf(m.agent,sizeof(m.agent),"%s",agent);
        snprintf(m.agent_status,sizeof(m.agent_status),"%s",start?"Executando":success?"Concluido":"Falhou");
        m.tokens_known=known; m.input_tokens=in; m.output_tokens=out;
        m.agent_running=start; m.connected=true; m.last_seen=now; m.demo=false;
        if (!start) tb_request(&b,fail?TB_ERROR:TB_SUCCESS,now,5000);
        char entry[48]; snprintf(entry,sizeof(entry),"%.31s: %s",agent,start?"iniciou":success?"concluiu":"falhou");
        tb_add_event(&m,entry);
    } else {reply.error="unknown_type"; goto done;}
    *model=m; *behavior=b; tb_model_tick(model,behavior,now); reply.error=NULL;
done:
    cJSON_Delete(o); return reply;
}
