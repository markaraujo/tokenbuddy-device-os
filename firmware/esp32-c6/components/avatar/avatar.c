#include "avatar.h"
#include "lvgl.h"
static lv_obj_t *face, *status;
static tb_pose current;

static void draw_face(lv_event_t *event)
{
    lv_draw_ctx_t *ctx=lv_event_get_draw_ctx(event);
    lv_area_t area; lv_obj_get_coords(face,&area);
    lv_color_t color=lv_color_hex(current.state==TB_ERROR?0xff5252:0x00e5ff);
    for (int eye=0; eye<2; ++eye) {
        int x=area.x1+49+eye*74+current.gaze, y=area.y1+55;
        if (current.state==TB_ERROR && !current.blink) {
            lv_draw_line_dsc_t line; lv_draw_line_dsc_init(&line);
            line.color=color; line.width=8; line.round_start=1; line.round_end=1;
            lv_point_t a={x-13,y-14},b={x+13,y+14},c={x-13,y+14},d={x+13,y-14};
            lv_draw_line(ctx,&line,&a,&b); lv_draw_line(ctx,&line,&c,&d);
        } else if (current.state==TB_HAPPY && !current.blink) {
            lv_draw_arc_dsc_t arc; lv_draw_arc_dsc_init(&arc);
            arc.color=color; arc.width=10; arc.rounded=1;
            lv_point_t center={x,y+10};
            lv_draw_arc(ctx,&arc,&center,21,190,350);
        } else {
            int height=(current.state==TB_THINKING?14:32)*current.openness/100;
            if (height<3) height=3;
            lv_area_t eye_area={x-21,y-height/2,x+21,y+(height-1)/2};
            lv_draw_rect_dsc_t rect; lv_draw_rect_dsc_init(&rect);
            rect.bg_color=color; rect.radius=height/2;
            rect.shadow_color=color; rect.shadow_width=10; rect.shadow_opa=LV_OPA_30;
            lv_draw_rect(ctx,&rect,&eye_area);
        }
    }
}
void avatar_init(void)
{
    lv_obj_t *screen=lv_scr_act();
    lv_obj_set_style_bg_color(screen,lv_color_hex(0x020a12),0);
    lv_obj_clear_flag(screen,LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_t *title=lv_label_create(screen);
    lv_label_set_text(title,"TokenBuddy");
    lv_obj_set_style_text_color(title,lv_color_hex(0x00e5ff),0);
    lv_obj_align(title,LV_ALIGN_TOP_MID,0,22);
    face=lv_obj_create(screen); lv_obj_remove_style_all(face);
    lv_obj_set_size(face,172,120); lv_obj_set_pos(face,0,88);
    lv_obj_clear_flag(face,LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_add_event_cb(face,draw_face,LV_EVENT_DRAW_MAIN,NULL);
    lv_obj_t *greeting=lv_label_create(screen);
    lv_label_set_text(greeting,"Ola, humano!");
    lv_obj_set_style_text_color(greeting,lv_color_hex(0xf0f6ff),0);
    lv_obj_align(greeting,LV_ALIGN_BOTTOM_MID,0,-65);
    status=lv_label_create(screen); lv_label_set_text(status,"REPOUSO");
    lv_obj_set_style_text_color(status,lv_color_hex(0x91a5b8),0);
    lv_obj_align(status,LV_ALIGN_BOTTOM_MID,0,-34);
    lv_obj_t *demo=lv_label_create(screen); lv_label_set_text(demo,"DEMO LOCAL");
    lv_obj_set_style_text_color(demo,lv_color_hex(0x526779),0);
    lv_obj_align(demo,LV_ALIGN_BOTTOM_MID,0,-12);
}
void avatar_update(tb_pose pose)
{
    bool changed=current.state!=pose.state;
    current=pose;
    if (changed) {
        lv_label_set_text(status,tb_state_name(pose.state));
        lv_obj_align(status,LV_ALIGN_BOTTOM_MID,0,-34);
    }
    lv_obj_invalidate(face);
}
