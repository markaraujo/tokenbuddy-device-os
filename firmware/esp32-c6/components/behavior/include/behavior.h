#pragma once
#include <stdint.h>
#include <stdbool.h>
typedef enum {TB_IDLE, TB_HAPPY, TB_THINKING, TB_ERROR, TB_CURIOUS, TB_WORKING,
    TB_SUCCESS, TB_SLEEPING, TB_DISCONNECTED, TB_DIZZY, TB_STATE_COUNT} tb_state;
typedef struct {tb_state state, base; uint32_t since, duration;} tb_behavior;
typedef struct {tb_state state; int gaze, openness; bool blink;} tb_pose;
void tb_init(tb_behavior *b, uint32_t now);
void tb_set_base(tb_behavior *b, tb_state state);
bool tb_request(tb_behavior *b, tb_state state, uint32_t now, uint32_t duration);
tb_pose tb_sample(tb_behavior *b, uint32_t now);
const char *tb_state_name(tb_state state);
