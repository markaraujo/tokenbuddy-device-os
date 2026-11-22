#include <inttypes.h>
#include "ST7789.h"
#include "LVGL_Driver.h"
#include "esp_system.h"

static void next_pattern(lv_timer_t *timer)
{
    static unsigned step;
    static const uint32_t colors[] = {0xff0000, 0x00ff00, 0x0000ff, 0xffffff, 0x000000};
    (void)timer;
    lv_obj_set_style_bg_color(lv_scr_act(), lv_color_hex(colors[step % 5]), 0);
    lv_obj_set_style_bg_opa(lv_scr_act(), LV_OPA_COVER, 0);
    ESP_LOGI("lcd_test", "pattern=%u free_heap=%" PRIu32, step % 5, esp_get_free_heap_size());
    ++step;
}

void app_main(void)
{
    ESP_LOGW("lcd_test", "Candidate non-touch ST7789; physical identification required");
    const spi_bus_config_t bus = {
        .sclk_io_num = EXAMPLE_PIN_NUM_SCLK,
        .mosi_io_num = EXAMPLE_PIN_NUM_MOSI,
        .miso_io_num = -1,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
        .max_transfer_sz = EXAMPLE_LCD_H_RES * 40 * sizeof(uint16_t),
    };
    /* Official non-touch schematic: keep the shared microSD deselected. */
    ESP_ERROR_CHECK(gpio_set_direction(GPIO_NUM_4, GPIO_MODE_OUTPUT));
    ESP_ERROR_CHECK(gpio_set_level(GPIO_NUM_4, 1));
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &bus, SPI_DMA_CH_AUTO));
    LCD_Init();
    BK_Light(30);
    LVGL_Init();
    lv_obj_t *border = lv_obj_create(lv_scr_act());
    lv_obj_set_size(border, 172, 320);
    lv_obj_set_pos(border, 0, 0);
    lv_obj_clear_flag(border, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_style_radius(border, 0, 0);
    lv_obj_set_style_bg_opa(border, LV_OPA_TRANSP, 0);
    lv_obj_set_style_border_color(border, lv_color_hex(0x808080), 0);
    lv_obj_set_style_border_width(border, 2, 0);
    lv_obj_t *label = lv_label_create(lv_scr_act());
    lv_label_set_text(label, "TOP\nTokenBuddy\n172 x 320\nLCD TEST");
    lv_obj_set_style_text_align(label, LV_TEXT_ALIGN_CENTER, 0);
    lv_obj_set_style_text_color(label, lv_color_hex(0x00e5ff), 0);
    lv_obj_set_style_bg_color(label, lv_color_black(), 0);
    lv_obj_set_style_bg_opa(label, LV_OPA_COVER, 0);
    lv_obj_center(label);
    next_pattern(NULL);
    lv_timer_create(next_pattern, 3000, NULL);
    for (;;) {
        lv_timer_handler();
        vTaskDelay(pdMS_TO_TICKS(10));
    }
}
