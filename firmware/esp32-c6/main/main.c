#include <inttypes.h>
#include "esp_chip_info.h"
#include "esp_idf_version.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_timer.h"
#include "display.h"
#include "board.h"
#include "avatar.h"
#include "behavior.h"
#include <stdlib.h>

bool behavior_tests(void);

void app_main(void)
{
    esp_chip_info_t chip;
    esp_chip_info(&chip);
    ESP_LOGI("tokenbuddy", "TokenBuddy Device OS 0.2.0 | IDF %s", esp_get_idf_version());
    ESP_LOGI("tokenbuddy", "Chip model=%d revision=%u cores=%u",
             (int)chip.model, (unsigned)chip.revision, (unsigned)chip.cores);
    if (!behavior_tests()) {ESP_LOGE("tokenbuddy","Behavior self-test FAILED"); abort();}
    ESP_LOGI("tokenbuddy","Behavior self-test PASS");
    display_init();
    avatar_init();
    tb_behavior behavior;
    uint32_t now=(uint32_t)(esp_timer_get_time()/1000);
    tb_init(&behavior,now);
    avatar_update(tb_sample(&behavior,now));
    display_process();
    board_brightness(30);
    uint32_t next_demo=now, last_log=now;
    unsigned demo=0;
    tb_state previous=TB_STATE_COUNT;
    for (;;) {
        now=(uint32_t)(esp_timer_get_time()/1000);
        if ((uint32_t)(now-next_demo)>=6000) {
            next_demo=now;
            demo=(demo+1)%TB_STATE_COUNT;
            tb_request(&behavior,(tb_state)demo,now,6000);
        }
        tb_pose pose=tb_sample(&behavior,now);
        avatar_update(pose);
        if (pose.state!=previous) {
            previous=pose.state;
            board_rgb(pose.state==TB_ERROR?14:0,pose.state==TB_HAPPY?10:3,pose.state==TB_ERROR?0:9);
            ESP_LOGI("tokenbuddy","demo state=%s",tb_state_name(pose.state));
        }
        display_process();
        if ((uint32_t)(now-last_log)>=5000) {
            last_log=now;
            ESP_LOGI("tokenbuddy","alive | selftest=PASS | free_heap=%" PRIu32,esp_get_free_heap_size());
        }
        vTaskDelay(pdMS_TO_TICKS(30));
    }
}
