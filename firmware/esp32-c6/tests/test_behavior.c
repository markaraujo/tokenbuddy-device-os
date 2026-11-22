#include "behavior.h"
#include <stdint.h>
#include <stdbool.h>
#define CHECK(condition) do {if (!(condition)) return false;} while (0)
bool behavior_tests(void)
{
    tb_behavior b; tb_init(&b,0);
    CHECK(tb_sample(&b,100).state==TB_IDLE);
    CHECK(tb_request(&b,TB_ERROR,100,200));
    CHECK(!tb_request(&b,TB_HAPPY,150,100));
    CHECK(tb_sample(&b,299).state==TB_ERROR);
    CHECK(tb_sample(&b,300).state==TB_IDLE);
    CHECK(tb_request(&b,TB_HAPPY,300,100));
    CHECK(tb_request(&b,TB_THINKING,310,100));
    CHECK(!tb_request(&b,TB_STATE_COUNT,320,100));
    CHECK(!tb_request(&b,TB_ERROR,320,0));
    tb_init(&b,UINT32_MAX-15);
    CHECK(tb_request(&b,TB_ERROR,UINT32_MAX-15,20));
    CHECK(tb_sample(&b,3).state==TB_ERROR);
    CHECK(tb_sample(&b,4).state==TB_IDLE);
    tb_init(&b,0);
    CHECK(tb_sample(&b,80).openness==0);
    CHECK(tb_sample(&b,160).openness==100);
    for (uint32_t t=0;t<20000;t+=17) {
        tb_pose p=tb_sample(&b,t);
        CHECK(p.openness>=0 && p.openness<=100);
        CHECK(p.gaze>=-6 && p.gaze<=6);
    }
    return true;
}
#ifdef TOKENBUDDY_HOST_TEST
int main(void) {return behavior_tests()?0:1;}
#endif
