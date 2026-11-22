#pragma once
#include "behavior.h"
#include <stddef.h>
#define TB_PAGES 5
#define TB_EVENTS 4
typedef struct {
    unsigned page, brightness, animation;
    bool demo, connected, agent_running;
    uint32_t last_seen;
    char agent[32], agent_status[24];
    bool tokens_known, gpu_known, telemetry_known;
    uint32_t input_tokens, output_tokens;
    double cpu, ram, ram_used, ram_total, gpu, vram_used, vram_total;
    char events[TB_EVENTS][48];
    unsigned event_count, event_head;
} tb_model;
typedef struct {bool raw, stable, active; uint32_t changed, pressed;} tb_button;
typedef enum {TB_BUTTON_NONE, TB_BUTTON_SHORT, TB_BUTTON_LONG} tb_button_event;
typedef struct {int id; const char *error;} tb_reply;
void tb_model_init(tb_model *m);
void tb_model_tick(tb_model *m, tb_behavior *b, uint32_t now);
void tb_add_event(tb_model *m, const char *text);
tb_button_event tb_button_poll(tb_button *button, bool pressed, uint32_t now);
void tb_button_action(tb_model *m, tb_behavior *b, tb_button_event event, uint32_t now);
tb_reply tb_command(tb_model *m, tb_behavior *b, const char *line, uint32_t now);
