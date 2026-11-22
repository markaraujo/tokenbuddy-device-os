#include "ui.h"
#include "avatar.h"
#include "lvgl.h"
#include <stdio.h>
#include <string.h>
#include <inttypes.h>

typedef struct {lv_obj_t *root,*caption,*value,*detail,*bar,*marker;int target;} card;
static lv_obj_t *page_root,*title,*subtitle,*icon,*status_dot,*hint,*version,*beam,*dots[5];
static card cards[4];
static unsigned previous_page=99;
static uint32_t refreshed;
static lv_color_t cyan(void) {return lv_color_hex(0x00e5ff);}
static void visible(lv_obj_t *object,bool show)
{
    if (show) lv_obj_clear_flag(object,LV_OBJ_FLAG_HIDDEN);
    else lv_obj_add_flag(object,LV_OBJ_FLAG_HIDDEN);
}
static void text(lv_obj_t *label,const char *value)
{
    if (strcmp(lv_label_get_text(label),value)) lv_label_set_text(label,value);
}
static lv_obj_t *plain(lv_obj_t *parent)
{
    lv_obj_t *object=lv_obj_create(parent);lv_obj_remove_style_all(object);
    lv_obj_clear_flag(object,LV_OBJ_FLAG_SCROLLABLE);lv_obj_clear_flag(object,LV_OBJ_FLAG_CLICKABLE);
    return object;
}
static lv_obj_t *label(lv_obj_t *parent,const lv_font_t *font,uint32_t color)
{
    lv_obj_t *object=lv_label_create(parent);lv_label_set_text(object,"");
    lv_obj_set_style_text_font(object,font,0);lv_obj_set_style_text_color(object,lv_color_hex(color),0);
    lv_obj_set_style_text_opa(object,LV_OPA_COVER,0);
    return object;
}
static void show_card(unsigned i,int x,int y,int w,int h)
{
    card *c=&cards[i];visible(c->root,true);lv_obj_set_pos(c->root,x,y);lv_obj_set_size(c->root,w,h);
    visible(c->caption,true);visible(c->value,true);visible(c->detail,true);visible(c->bar,false);visible(c->marker,false);
    lv_obj_set_pos(c->caption,10,7);lv_obj_set_size(c->caption,w-20,16);
    lv_obj_set_pos(c->value,10,25);lv_obj_set_size(c->value,w-20,28);
    lv_obj_set_style_text_font(c->value,&lv_font_montserrat_18,0);
    lv_obj_set_style_text_color(c->value,lv_color_white(),0);
    lv_obj_set_style_text_line_space(c->value,2,0);
    lv_obj_set_pos(c->detail,10,54);lv_obj_set_size(c->detail,w-20,16);
    lv_obj_set_style_text_font(c->detail,&lv_font_montserrat_12,0);
    lv_obj_set_style_bg_opa(c->detail,LV_OPA_TRANSP,0);
    lv_obj_set_style_pad_all(c->detail,0,0);
    lv_obj_set_style_text_color(c->detail,lv_color_hex(0xc5dfed),0);
    lv_obj_set_pos(c->bar,10,h-10);lv_obj_set_size(c->bar,w-20,4);
    lv_obj_set_style_border_color(c->root,lv_color_hex(0x163c50),0);
    c->target=-1;
}
static void bar_value(unsigned i,int value,bool known,uint32_t color)
{
    card *c=&cards[i];visible(c->bar,true);
    if (value<0) value=0;
    if (value>100) value=100;
    if (!known) value=0;
    lv_obj_set_style_bg_color(c->bar,lv_color_hex(color),LV_PART_INDICATOR);
    if (c->target!=value) {lv_bar_set_value(c->bar,value,known?LV_ANIM_ON:LV_ANIM_OFF);c->target=value;}
}
static void compact(char *buffer,size_t size,uint32_t value)
{
    if (value>=1000000) snprintf(buffer,size,"%.1fM",value/1000000.0);
    else if (value>=10000) snprintf(buffer,size,"%.0fk",value/1000.0);
    else if (value>=1000) snprintf(buffer,size,"%.1fk",value/1000.0);
    else snprintf(buffer,size,"%" PRIu32,value);
}
static void layout(unsigned page)
{
    for (unsigned i=0;i<4;++i) visible(cards[i].root,false);
    visible(beam,false);visible(version,page==4);
    if (page==1) {
        show_card(0,10,66,152,91);
        show_card(1,10,169,73,80);show_card(2,89,169,73,80);
        for (unsigned i=1;i<3;++i) {lv_obj_set_style_text_font(cards[i].value,&lv_font_montserrat_22,0);lv_obj_set_style_text_color(cards[i].value,i==1?cyan():lv_color_hex(0x36edb5),0);}
        text(cards[1].caption,"ENTRADA");text(cards[2].caption,"SAIDA");
        lv_obj_set_pos(cards[0].detail,10,54);lv_obj_set_size(cards[0].detail,132,20);
        lv_obj_set_style_text_font(cards[0].detail,&lv_font_montserrat_12,0);
        lv_obj_set_style_pad_hor(cards[0].detail,5,0);lv_obj_set_style_pad_ver(cards[0].detail,2,0);
        lv_obj_set_style_radius(cards[0].detail,5,0);
    } else if (page==2 || page==3) {
        for (unsigned i=0;i<4;++i) {
            show_card(i,10,65+49*i,152,45);
            if (page==2) {
                lv_obj_set_size(cards[i].caption,44,16);
                lv_obj_set_pos(cards[i].value,57,3);lv_obj_set_size(cards[i].value,85,24);
                lv_obj_set_style_text_font(cards[i].value,i==3?&lv_font_montserrat_16:&lv_font_montserrat_18,0);
                lv_obj_set_style_text_align(cards[i].value,LV_TEXT_ALIGN_RIGHT,0);
                lv_obj_set_pos(cards[i].detail,10,24);lv_obj_set_size(cards[i].detail,132,16);
                lv_obj_set_pos(cards[i].bar,10,41);lv_obj_set_size(cards[i].bar,132,3);
            } else {
                visible(cards[i].caption,false);visible(cards[i].detail,false);
                visible(cards[i].marker,true);
                lv_obj_set_pos(cards[i].value,12,5);lv_obj_set_size(cards[i].value,129,36);
                lv_obj_set_style_text_font(cards[i].value,&lv_font_montserrat_14,0);
                lv_obj_set_style_text_align(cards[i].value,LV_TEXT_ALIGN_LEFT,0);
            }
        }
    } else if (page==4) {
        show_card(0,10,66,152,82);text(cards[0].caption,"BRILHO");
        lv_obj_set_style_text_font(cards[0].value,&lv_font_montserrat_22,0);lv_obj_set_size(cards[0].value,64,28);
        lv_obj_set_pos(cards[0].detail,76,33);lv_obj_set_size(cards[0].detail,66,16);text(cards[0].detail,"MAX 50%");
        lv_obj_set_pos(cards[0].bar,10,65);lv_obj_set_size(cards[0].bar,132,5);
        show_card(1,10,160,152,40);show_card(2,10,208,152,40);
        for (unsigned i=1;i<3;++i) {
            lv_obj_set_size(cards[i].caption,76,16);lv_obj_set_pos(cards[i].caption,10,12);
            lv_obj_set_pos(cards[i].value,84,10);lv_obj_set_size(cards[i].value,58,22);
            lv_obj_set_style_text_font(cards[i].value,&lv_font_montserrat_14,0);
            visible(cards[i].detail,false);
        }
        text(cards[1].caption,"ANIMACAO");text(cards[2].caption,"DEMO LOCAL");
    }
    /* Card slots are reused; reset alignment inherited from the metrics page. */
    if (page!=2) for (unsigned i=0;i<4;++i) lv_obj_set_style_text_align(cards[i].value,LV_TEXT_ALIGN_LEFT,0);
}
void ui_init(void)
{
    lv_obj_set_style_bg_color(lv_scr_act(),lv_color_hex(0x030a15),0);
    lv_obj_set_style_bg_opa(lv_scr_act(),LV_OPA_COVER,0);
    avatar_init();
    page_root=plain(lv_scr_act());lv_obj_set_size(page_root,172,320);
    lv_obj_set_style_bg_color(page_root,lv_color_hex(0x030a15),0);
    lv_obj_set_style_bg_grad_color(page_root,lv_color_hex(0x071727),0);
    lv_obj_set_style_bg_grad_dir(page_root,LV_GRAD_DIR_VER,0);
    lv_obj_set_style_bg_opa(page_root,LV_OPA_COVER,0);
    lv_obj_t *badge=plain(page_root);lv_obj_set_pos(badge,13,19);lv_obj_set_size(badge,29,29);
    lv_obj_set_style_radius(badge,9,0);lv_obj_set_style_bg_color(badge,lv_color_hex(0x0b293b),0);lv_obj_set_style_bg_opa(badge,LV_OPA_COVER,0);
    lv_obj_set_style_border_color(badge,lv_color_hex(0x1a6175),0);lv_obj_set_style_border_width(badge,1,0);
    icon=label(badge,&lv_font_montserrat_14,0x4befff);lv_obj_center(icon);
    title=label(page_root,&lv_font_montserrat_16,0xffffff);lv_obj_set_pos(title,51,20);
    subtitle=label(page_root,&lv_font_montserrat_12,0xc5dfed);lv_obj_set_pos(subtitle,52,40);
    status_dot=plain(page_root);lv_obj_set_pos(status_dot,151,25);lv_obj_set_size(status_dot,4,4);
    lv_obj_set_style_radius(status_dot,2,0);lv_obj_set_style_bg_opa(status_dot,LV_OPA_COVER,0);
    lv_obj_t *rule=plain(page_root);lv_obj_set_pos(rule,13,57);lv_obj_set_size(rule,146,1);
    lv_obj_set_style_bg_color(rule,lv_color_hex(0x1b4155),0);lv_obj_set_style_bg_opa(rule,LV_OPA_60,0);
    for (unsigned i=0;i<4;++i) {
        card *c=&cards[i];c->root=plain(page_root);
        lv_obj_set_style_radius(c->root,11,0);lv_obj_set_style_bg_color(c->root,lv_color_hex(0x0b1d30),0);
        lv_obj_set_style_bg_grad_color(c->root,lv_color_hex(0x071423),0);lv_obj_set_style_bg_grad_dir(c->root,LV_GRAD_DIR_VER,0);
        lv_obj_set_style_bg_opa(c->root,LV_OPA_COVER,0);lv_obj_set_style_border_width(c->root,1,0);
        lv_obj_set_style_border_color(c->root,lv_color_hex(0x163c50),0);
        lv_obj_set_style_shadow_color(c->root,lv_color_hex(0x00b7da),0);
        lv_obj_set_style_shadow_width(c->root,7,0);lv_obj_set_style_shadow_opa(c->root,LV_OPA_10,0);
        c->caption=label(c->root,&lv_font_montserrat_12,0xc5dfed);
        c->value=label(c->root,&lv_font_montserrat_18,0xffffff);lv_label_set_long_mode(c->value,LV_LABEL_LONG_DOT);
        c->detail=label(c->root,&lv_font_montserrat_12,0xc5dfed);
        c->bar=lv_bar_create(c->root);lv_obj_remove_style_all(c->bar);
        lv_obj_set_style_radius(c->bar,3,LV_PART_MAIN);lv_obj_set_style_radius(c->bar,3,LV_PART_INDICATOR);
        lv_obj_set_style_bg_color(c->bar,lv_color_hex(0x18344a),LV_PART_MAIN);
        lv_obj_set_style_bg_opa(c->bar,LV_OPA_COVER,LV_PART_MAIN);lv_obj_set_style_bg_opa(c->bar,LV_OPA_COVER,LV_PART_INDICATOR);
        lv_obj_set_style_anim_time(c->bar,350,0);lv_bar_set_range(c->bar,0,100);
        c->marker=plain(c->root);lv_obj_set_pos(c->marker,0,10);lv_obj_set_size(c->marker,3,23);
        lv_obj_set_style_radius(c->marker,2,0);lv_obj_set_style_bg_opa(c->marker,LV_OPA_COVER,0);
    }
    beam=plain(cards[0].root);lv_obj_set_size(beam,26,3);lv_obj_set_style_radius(beam,2,0);
    lv_obj_set_style_bg_color(beam,cyan(),0);lv_obj_set_style_bg_opa(beam,LV_OPA_COVER,0);
    lv_obj_set_style_shadow_color(beam,cyan(),0);lv_obj_set_style_shadow_width(beam,5,0);lv_obj_set_style_shadow_opa(beam,LV_OPA_30,0);
    hint=label(page_root,&lv_font_montserrat_12,0xc5dfed);lv_obj_align(hint,LV_ALIGN_TOP_MID,0,280);
    version=label(page_root,&lv_font_montserrat_12,0xb7d6e6);text(version,"v0.3.0  |  8 MB");lv_obj_align(version,LV_ALIGN_TOP_MID,0,258);
    for (unsigned i=0;i<5;++i) {
        dots[i]=plain(page_root);lv_obj_set_style_radius(dots[i],2,0);lv_obj_set_style_bg_opa(dots[i],LV_OPA_COVER,0);
    }
    /* The same thin frame and safe margins refine all five screens. */
    lv_obj_t *frame=plain(lv_scr_act());lv_obj_set_pos(frame,3,3);lv_obj_set_size(frame,166,314);
    lv_obj_set_style_radius(frame,15,0);lv_obj_set_style_border_width(frame,1,0);
    lv_obj_set_style_border_color(frame,lv_color_hex(0x246277),0);lv_obj_set_style_border_opa(frame,LV_OPA_60,0);
}
void ui_update(const tb_model *m,tb_pose pose,uint32_t now)
{
    if (m->animation==0) {pose.gaze=0;pose.openness=100;pose.blink=false;}
    else if (m->animation==2) pose.gaze=pose.gaze*3/2;
    bool changed=m->page!=previous_page;
    if (changed) {
        previous_page=m->page;avatar_visible(m->page==0);visible(page_root,m->page!=0);
        layout(m->page);
        lv_obj_set_style_opa(page_root,LV_OPA_COVER,0);
        static const char *titles[]={"TokenBuddy","Agentes","Sistema","Eventos","Ajustes"};
        static const char *icons[]={LV_SYMBOL_HOME,LV_SYMBOL_PLAY,LV_SYMBOL_CHARGE,LV_SYMBOL_BELL,LV_SYMBOL_SETTINGS};
        text(title,titles[m->page]);text(icon,icons[m->page]);lv_obj_center(icon);
        text(hint,m->page==4?"SEGURE BOOT: BRILHO":"BOOT: PROXIMA TELA");lv_obj_align(hint,LV_ALIGN_TOP_MID,0,280);
        for (unsigned i=0;i<5;++i) {
            lv_obj_set_pos(dots[i],51+16*i-(m->page==i?4:0),302);lv_obj_set_size(dots[i],m->page==i?12:4,4);
            lv_obj_set_style_bg_color(dots[i],m->page==i?cyan():lv_color_hex(0x28465d),0);
        }
    }
    if (m->page==0) {avatar_update(pose);avatar_context(m->connected,m->demo);return;}
    if (m->page==1 && m->agent_running) {
        visible(beam,true);lv_obj_set_pos(beam,10+(now/14)%106,80);
    } else visible(beam,false);
    if (!changed && (uint32_t)(now-refreshed)<500) return;
    refreshed=now;
    text(subtitle,m->connected?"USB ONLINE":"USB OFFLINE");
    lv_obj_set_style_bg_color(status_dot,lv_color_hex(m->connected?0x32eeb0:0xf6b75a),0);
    char value[96],detail[96];
    if (m->page==1) {
        text(cards[0].caption,m->agent_running?"AGENTE ATIVO":"ULTIMA TAREFA");
        text(cards[0].value,m->agent[0]?m->agent:"Sem tarefa");text(cards[0].detail,m->agent_status);
        uint32_t color=!strcmp(m->agent_status,"Falhou")?0xff5367:!strcmp(m->agent_status,"Concluido")?0x30eeb0:0x00e5ff;
        lv_obj_set_style_text_color(cards[0].detail,lv_color_hex(color),0);
        lv_obj_set_style_bg_color(cards[0].detail,lv_color_hex(color),0);lv_obj_set_style_bg_opa(cards[0].detail,LV_OPA_10,0);
        lv_obj_set_style_border_color(cards[0].root,lv_color_hex(color),0);
        for (unsigned i=1;i<3;++i) {
            if (m->tokens_known) compact(value,sizeof(value),i==1?m->input_tokens:m->output_tokens);else snprintf(value,sizeof(value),"--");
            text(cards[i].value,value);text(cards[i].detail,m->tokens_known?"TOKENS":"N/D");
        }
    } else if (m->page==2) {
        const char *names[]={"CPU","RAM","GPU","VRAM"};
        uint32_t colors[]={0x00e5ff,0x30eeb0,0xa18bff,0x46cfff};
        for (unsigned i=0;i<4;++i) {
            bool known=m->connected && m->telemetry_known && (i<2 || m->gpu_known);
            double percent=i==0?m->cpu:i==1?m->ram:i==2?m->gpu:(m->vram_total>0?m->vram_used/m->vram_total*100:0);
            text(cards[i].caption,names[i]);
            if (!known) {snprintf(value,sizeof(value),"--");snprintf(detail,sizeof(detail),m->connected?"DADOS INDISPONIVEIS":"HOST DESCONECTADO");}
            else if (i==1) {snprintf(value,sizeof(value),"%.0f%%",percent);snprintf(detail,sizeof(detail),"%.1f / %.1f GiB",m->ram_used/1024,m->ram_total/1024);}
            else if (i==3) {snprintf(value,sizeof(value),"%.0f MiB",m->vram_used);snprintf(detail,sizeof(detail),"DE %.0f MiB",m->vram_total);}
            else {snprintf(value,sizeof(value),"%.0f%%",percent);snprintf(detail,sizeof(detail),i==0?"PROCESSADOR":"NVIDIA GPU");}
            text(cards[i].value,value);text(cards[i].detail,detail);bar_value(i,(int)(percent+.5),known,colors[i]);
        }
    } else if (m->page==3) {
        for (unsigned i=0;i<4;++i) {
            visible(cards[i].root,i<m->event_count);
            if (i>=m->event_count) continue;
            unsigned index=(m->event_head+TB_EVENTS-1-i)%TB_EVENTS;
            const char *entry=m->events[index];
            const char *separator=strstr(entry,": ");
            if (separator) {
                int length=(int)(separator-entry);
                snprintf(value,sizeof(value),"%.*s%s\n%s",length>8?8:length,entry,length>8?"...":"",separator+2);
                text(cards[i].value,value);
            } else text(cards[i].value,entry);
            uint32_t color=strstr(entry,"falhou")?0xff5367:strstr(entry,"concluiu")?0x30eeb0:strstr(entry,"Host")?0xf6b75a:0x00e5ff;
            lv_obj_set_style_bg_color(cards[i].marker,lv_color_hex(color),0);
        }
    } else {
        snprintf(value,sizeof(value),"%u%%",m->brightness);text(cards[0].value,value);
        bar_value(0,m->brightness,true,0x00e5ff);
        text(cards[1].value,m->animation==0?"Parada":m->animation==1?"Normal":"Intensa");
        text(cards[2].value,m->demo?"Ativa":"Off");
        lv_obj_set_style_text_color(cards[2].value,lv_color_hex(m->demo?0x30eeb0:0xc5dfed),0);
    }
}
