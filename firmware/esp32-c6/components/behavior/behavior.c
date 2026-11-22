#include "behavior.h"
static unsigned priority(tb_state s) {return s==TB_ERROR?3:s==TB_THINKING?2:s==TB_HAPPY?1:0;}
void tb_init(tb_behavior *b, uint32_t now) {*b=(tb_behavior){TB_IDLE,now,0};}
tb_pose tb_sample(tb_behavior *b, uint32_t now)
{
    if (b->duration && (uint32_t)(now-b->since)>=b->duration) tb_init(b,now);
    unsigned phase=now%4200;
    unsigned openness=phase<80?100-phase*100/80:phase<160?(phase-80)*100/80:100;
    unsigned look=now%6000;
    int gaze=look<3000?(int)(look*12/3000)-6:6-(int)((look-3000)*12/3000);
    return (tb_pose){b->state,gaze,(int)openness,phase<160};
}
bool tb_request(tb_behavior *b, tb_state state, uint32_t now, uint32_t duration)
{
    tb_sample(b,now);
    if ((unsigned)state>=TB_STATE_COUNT || duration==0 || priority(state)<priority(b->state)) return false;
    *b=(tb_behavior){state,now,duration};
    return true;
}
const char *tb_state_name(tb_state state)
{
    static const char *names[]={"REPOUSO","FELIZ","CONCENTRADO","ERRO"};
    return (unsigned)state<TB_STATE_COUNT?names[state]:"INVALIDO";
}
