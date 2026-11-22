#include "avatar.h"
#include "lvgl.h"
#include <string.h>
static lv_obj_t *root,*face,*status,*connection;
static tb_pose current;
static tb_state from_state;
static uint32_t transition_at,last_update;
static int gaze_fixed;
static uint32_t clock_ms;

static lv_color_t color_for(tb_state state)
{
    return lv_color_hex(state==TB_ERROR?0xff5367:state==TB_SUCCESS?0x20efac:
        state==TB_SLEEPING?0x265b70:state==TB_DISCONNECTED?0x508ca0:0x00e5ff);
}
static void neon_arc(lv_draw_ctx_t *ctx,lv_point_t center,unsigned radius,
    unsigned start,unsigned end,lv_color_t color,unsigned opacity,unsigned width)
{
    lv_draw_arc_dsc_t arc;lv_draw_arc_dsc_init(&arc);
    arc.color=color;arc.rounded=1;
    arc.width=width+5;arc.opa=opacity/7;lv_draw_arc(ctx,&arc,&center,radius,start,end);
    arc.width=width;arc.opa=opacity;lv_draw_arc(ctx,&arc,&center,radius,start,end);
}
static void rounded_eye(lv_draw_ctx_t *ctx,int x,int y,int height,lv_color_t color,unsigned opacity)
{
    if (!opacity) return;
    if (height<3) height=3;
    lv_area_t box={x-21,y-height/2,x+21,y+(height-1)/2};
    lv_draw_rect_dsc_t rect;lv_draw_rect_dsc_init(&rect);
    rect.radius=height/2;rect.bg_color=color;rect.bg_opa=opacity;
    rect.shadow_color=color;rect.shadow_width=14;rect.shadow_opa=opacity/4;
    lv_draw_rect(ctx,&rect,&box);
    /* A narrow highlight gives the cyan core depth without bitmap assets. */
    if (height>12) {
        box.x1+=6;box.x2-=6;box.y1+=3;box.y2=box.y1+2;
        rect.shadow_width=0;rect.bg_color=lv_color_white();rect.bg_opa=opacity/7;
        lv_draw_rect(ctx,&rect,&box);
    }
}
static void draw_expression(lv_draw_ctx_t *ctx,const lv_area_t *area,tb_state state,unsigned opacity)
{
    if (!opacity) return;
    lv_color_t color=color_for(state);
    int open=current.openness;
    open=open*open*(300-2*open)/10000; /* Smoothstep eyelids. */
    int breathe=state==TB_SLEEPING?0:lv_trigo_sin((clock_ms%4800)*360/4800)*2/32768;
    for (int eye=0;eye<2;++eye) {
        int x=area->x1+49+eye*74+current.gaze,y=area->y1+35+breathe;
        unsigned shape_opa=opacity*(unsigned)open/100;
        if (state==TB_ERROR) {
            lv_draw_line_dsc_t line;lv_draw_line_dsc_init(&line);
            line.color=color;line.round_start=1;line.round_end=1;
            lv_point_t a={x-13,y-14},b={x+13,y+14},c={x-13,y+14},d={x+13,y-14};
            line.width=12;line.opa=shape_opa/7;
            lv_draw_line(ctx,&line,&a,&b);lv_draw_line(ctx,&line,&c,&d);
            line.width=7;line.opa=shape_opa;
            lv_draw_line(ctx,&line,&a,&b);lv_draw_line(ctx,&line,&c,&d);
        } else if (state==TB_HAPPY || state==TB_SUCCESS) {
            lv_point_t center={x,y+10};neon_arc(ctx,center,21,190,350,color,shape_opa,9);
        } else if (state==TB_CURIOUS) {
            lv_point_t center={x,y};neon_arc(ctx,center,18,0,359,color,shape_opa,6);
        } else if (state==TB_DIZZY) {
            lv_point_t center={x,y};
            for (unsigned i=0;i<3;++i) {
                unsigned start=(clock_ms/12+i*70)%360;
                neon_arc(ctx,center,6+i*6,start,(start+260)%360,color,shape_opa,3);
            }
        } else {
            int base=state==TB_SLEEPING?3:(state==TB_THINKING || state==TB_WORKING || state==TB_DISCONNECTED)?14:32;
            rounded_eye(ctx,x,y,base*open/100,color,opacity);
        }
        if (state==TB_ERROR || state==TB_HAPPY || state==TB_SUCCESS || state==TB_CURIOUS || state==TB_DIZZY)
            rounded_eye(ctx,x,y,3,color,opacity*(100-(unsigned)open)/100);
    }
    if (state==TB_WORKING || state==TB_THINKING) {
        for (unsigned i=0;i<3;++i) {
            int x=area->x1+71+15*i,y=area->y1+63;
            lv_area_t dot={x,y,x+3,y+3};lv_draw_rect_dsc_t rect;lv_draw_rect_dsc_init(&rect);
            rect.radius=2;rect.bg_color=color;rect.bg_opa=(clock_ms/250)%3==i?opacity:opacity/5;
            lv_draw_rect(ctx,&rect,&dot);
        }
    }
}
static void draw_face(lv_event_t *event)
{
    lv_draw_ctx_t *ctx=lv_event_get_draw_ctx(event);lv_area_t area;lv_obj_get_coords(face,&area);
    uint32_t elapsed=clock_ms-transition_at;
    unsigned mix=elapsed>=220?255:elapsed*255/220;
    if (mix<255) draw_expression(ctx,&area,from_state,255-mix);
    draw_expression(ctx,&area,current.state,mix);
}
void avatar_init(void)
{
    root=lv_obj_create(lv_scr_act());lv_obj_remove_style_all(root);
    lv_obj_set_size(root,172,320);lv_obj_clear_flag(root,LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_style_bg_color(root,lv_color_hex(0x030a15),0);
    lv_obj_set_style_bg_grad_color(root,lv_color_hex(0x071727),0);
    lv_obj_set_style_bg_grad_dir(root,LV_GRAD_DIR_VER,0);
    lv_obj_set_style_bg_opa(root,LV_OPA_COVER,0);
    lv_obj_t *title=lv_label_create(root);lv_label_set_text(title,"TokenBuddy");
    lv_obj_set_style_text_font(title,&lv_font_montserrat_18,0);
    lv_obj_set_style_text_color(title,lv_color_hex(0x7bf4ff),0);
    lv_obj_align(title,LV_ALIGN_TOP_MID,0,25);
    lv_obj_t *subtitle=lv_label_create(root);lv_label_set_text(subtitle,"SEU COMPANHEIRO");
    lv_obj_set_style_text_font(subtitle,&lv_font_montserrat_10,0);
    lv_obj_set_style_text_letter_space(subtitle,1,0);
    lv_obj_set_style_text_color(subtitle,lv_color_hex(0x5c839c),0);
    lv_obj_align(subtitle,LV_ALIGN_TOP_MID,0,52);
    face=lv_obj_create(root);lv_obj_remove_style_all(face);
    lv_obj_set_size(face,172,72);lv_obj_set_pos(face,0,108);
    lv_obj_clear_flag(face,LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_add_event_cb(face,draw_face,LV_EVENT_DRAW_MAIN,NULL);
    lv_obj_t *greeting=lv_label_create(root);lv_label_set_text(greeting,"Ola, humano!");
    lv_obj_set_style_text_font(greeting,&lv_font_montserrat_16,0);
    lv_obj_set_style_text_color(greeting,lv_color_hex(0xe4f2ff),0);
    lv_obj_align(greeting,LV_ALIGN_BOTTOM_MID,0,-83);
    status=lv_label_create(root);lv_label_set_text(status,"REPOUSO");
    lv_obj_set_style_text_font(status,&lv_font_montserrat_10,0);
    lv_obj_set_style_text_letter_space(status,1,0);
    lv_obj_set_style_text_color(status,lv_color_hex(0x00e5ff),0);
    lv_obj_set_style_bg_color(status,lv_color_hex(0x0b2639),0);
    lv_obj_set_style_bg_opa(status,LV_OPA_COVER,0);
    lv_obj_set_style_pad_hor(status,10,0);lv_obj_set_style_pad_ver(status,5,0);
    lv_obj_set_style_radius(status,10,0);lv_obj_align(status,LV_ALIGN_BOTTOM_MID,0,-44);
    connection=lv_label_create(root);lv_label_set_text(connection,"USB OFFLINE");
    lv_obj_set_style_text_font(connection,&lv_font_montserrat_10,0);
    lv_obj_set_style_text_color(connection,lv_color_hex(0x63859b),0);
    lv_obj_align(connection,LV_ALIGN_BOTTOM_MID,0,-20);
}
void avatar_visible(bool visible)
{
    if (visible) {lv_obj_clear_flag(root,LV_OBJ_FLAG_HIDDEN);lv_obj_fade_in(root,160,0);}
    else lv_obj_add_flag(root,LV_OBJ_FLAG_HIDDEN);
}
void avatar_context(bool online,bool demo)
{
    const char *text=demo?"DEMO LOCAL":online?"USB ONLINE":"USB OFFLINE";
    if (strcmp(lv_label_get_text(connection),text)) {
        lv_label_set_text(connection,text);lv_obj_align(connection,LV_ALIGN_BOTTOM_MID,0,-20);
    }
}
void avatar_update(tb_pose pose)
{
    clock_ms=lv_tick_get();unsigned dt=clock_ms-last_update;if (dt>100) dt=100;last_update=clock_ms;
    gaze_fixed+=(pose.gaze*256-gaze_fixed)*(int)dt/(80+(int)dt);
    pose.gaze=gaze_fixed/256;
    if (current.state!=pose.state) {
        from_state=current.state;transition_at=clock_ms;
        lv_label_set_text(status,tb_state_name(pose.state));
        lv_obj_set_style_text_color(status,color_for(pose.state),0);
        lv_obj_align(status,LV_ALIGN_BOTTOM_MID,0,-44);
    }
    current=pose;lv_obj_invalidate(face);
}
