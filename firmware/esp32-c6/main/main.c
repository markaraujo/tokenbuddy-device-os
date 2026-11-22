#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include "esp_log.h"
#include "esp_system.h"
#include "esp_timer.h"
#include "esp_mac.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "nvs_flash.h"
#include "nvs.h"
#include "board.h"
#include "display.h"
#include "control.h"
#include "ui.h"
#include "connectivity.h"

bool behavior_tests(void);
bool control_tests(void);
void app_main(void)
{
    if (!behavior_tests() || !control_tests()) {ESP_LOGE("tokenbuddy","selftest FAILED");abort();}
    ESP_LOGI("tokenbuddy","TokenBuddy 0.3.0 selftest PASS");
    tb_model model;tb_model_init(&model);
    tb_behavior behavior;tb_init(&behavior,0);
    tb_button button={0};
    nvs_handle_t settings=0;
    bool persistent=nvs_flash_init()==ESP_OK && nvs_open("tokenbuddy",NVS_READWRITE,&settings)==ESP_OK;
    uint8_t value;
    if (persistent && nvs_get_u8(settings,"brightness",&value)==ESP_OK && value<=50) model.brightness=value;
    if (persistent && nvs_get_u8(settings,"animation",&value)==ESP_OK && value<=2) model.animation=value;
    display_init();ui_init();link_init();
    /* C6 factory eFuse API returns EUI-64 (8 bytes); Wi-Fi STA API returns MAC-48. */
    uint8_t mac[6];ESP_ERROR_CHECK(esp_read_mac(mac,ESP_MAC_WIFI_STA));
    char mac_text[18];snprintf(mac_text,sizeof(mac_text),"%02x:%02x:%02x:%02x:%02x:%02x",mac[0],mac[1],mac[2],mac[3],mac[4],mac[5]);
    unsigned old_brightness=999,old_animation=model.animation;
    uint32_t last_status=0,last_demo=0,short_presses=0,long_presses=0;
    unsigned demo=0;
    char line[769],output[1024];
    unsigned reset_reason=(unsigned)esp_reset_reason();
    for (;;) {
        uint32_t now=(uint32_t)(esp_timer_get_time()/1000);
        tb_button_event event=tb_button_poll(&button,board_boot_pressed(),now);
        if (event==TB_BUTTON_SHORT) ++short_presses;
        if (event==TB_BUTTON_LONG) ++long_presses;
        tb_button_action(&model,&behavior,event,now);
        for (unsigned i=0;i<3 && link_read(line,sizeof(line));++i) {
            tb_reply reply=tb_command(&model,&behavior,line,now);
            snprintf(output,sizeof(output),"{\"v\":1,\"type\":\"ack\",\"id\":%d,\"ok\":%s,\"error\":\"%s\"}",reply.id,reply.error?"false":"true",reply.error?reply.error:"");
            link_write(output);
        }
        tb_model_tick(&model,&behavior,now);
        if (model.demo && (uint32_t)(now-last_demo)>=5000) {
            last_demo=now;demo=(demo+1)%TB_STATE_COUNT;
            tb_request(&behavior,(tb_state)demo,now,5000);
        }
        tb_pose pose=tb_sample(&behavior,now);
        ui_update(&model,pose,now);display_process();
        static tb_state previous=TB_STATE_COUNT;
        if (pose.state!=previous) {
            previous=pose.state;
            board_rgb(pose.state==TB_ERROR?14:0,pose.state==TB_SUCCESS?12:pose.state==TB_HAPPY?10:3,
                pose.state==TB_ERROR || pose.state==TB_SUCCESS?0:pose.state==TB_SLEEPING?1:8);
        }
        if (old_brightness!=model.brightness || old_animation!=model.animation) {
            board_brightness(model.brightness);
            old_brightness=model.brightness;old_animation=model.animation;
            if (persistent) {
                persistent=nvs_set_u8(settings,"brightness",model.brightness)==ESP_OK &&
                    nvs_set_u8(settings,"animation",model.animation)==ESP_OK && nvs_commit(settings)==ESP_OK;
            }
        }
        if ((uint32_t)(now-last_status)>=1000) {
            last_status=now;unsigned lv_free,frag;display_memory(&lv_free,&frag);
            snprintf(output,sizeof(output),"{\"v\":1,\"type\":\"status\",\"device\":\"TokenBuddy\",\"fw\":\"0.3.0\",\"mac\":\"%s\",\"reset_reason\":%u,\"uptime_ms\":%" PRIu32 ",\"page\":%u,\"state\":%u,\"online\":%s,\"brightness\":%u,\"animation\":%u,\"demo\":%s,\"heap\":%" PRIu32 ",\"lv_free\":%u,\"lv_frag\":%u,\"stack_min\":%u,\"short_presses\":%" PRIu32 ",\"long_presses\":%" PRIu32 ",\"settings_saved\":%s,\"selftest\":true}",mac_text,reset_reason,now,model.page,(unsigned)pose.state,model.connected?"true":"false",model.brightness,model.animation,model.demo?"true":"false",esp_get_free_heap_size(),lv_free,frag,(unsigned)uxTaskGetStackHighWaterMark(NULL),short_presses,long_presses,persistent?"true":"false");
            link_write(output);
        }
        vTaskDelay(pdMS_TO_TICKS(20));
    }
}
