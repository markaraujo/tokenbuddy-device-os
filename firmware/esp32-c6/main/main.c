#include <inttypes.h>
#include "esp_chip_info.h"
#include "esp_idf_version.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

void app_main(void)
{
    esp_chip_info_t chip;
    esp_chip_info(&chip);
    ESP_LOGI("tokenbuddy", "TokenBuddy Device OS 0.1.0 | IDF %s", esp_get_idf_version());
    ESP_LOGI("tokenbuddy", "Chip model=%d revision=%u cores=%u",
             (int)chip.model, (unsigned)chip.revision, (unsigned)chip.cores);
    for (;;) {
        ESP_LOGI("tokenbuddy", "alive | free_heap=%" PRIu32, esp_get_free_heap_size());
        vTaskDelay(pdMS_TO_TICKS(5000));
    }
}
