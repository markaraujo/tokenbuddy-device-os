#include "board.h"
#include "Vernon_ST7789T/Vernon_ST7789T.h"
#include "driver/gpio.h"
#include "driver/spi_master.h"
#include "driver/ledc.h"
#include "led_strip.h"

static led_strip_handle_t rgb;

void board_brightness(unsigned percent)
{
    if (percent > 50) percent = 50;
    ESP_ERROR_CHECK(ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0, 8191 * percent / 100));
    ESP_ERROR_CHECK(ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0));
}

void board_rgb(unsigned red, unsigned green, unsigned blue)
{
    ESP_ERROR_CHECK(led_strip_set_pixel(rgb, 0, red, green, blue));
    ESP_ERROR_CHECK(led_strip_refresh(rgb));
}

esp_lcd_panel_handle_t board_init(esp_lcd_panel_io_color_trans_done_cb_t done, void *context)
{
    /* Non-touch board: validated physically. SD shares SPI; never mount it here. */
    ESP_ERROR_CHECK(gpio_set_direction(GPIO_NUM_4, GPIO_MODE_OUTPUT));
    ESP_ERROR_CHECK(gpio_set_level(GPIO_NUM_4, 1));
    const ledc_timer_config_t timer = {.speed_mode=LEDC_LOW_SPEED_MODE,
        .duty_resolution=LEDC_TIMER_13_BIT, .timer_num=LEDC_TIMER_0,
        .freq_hz=5000, .clk_cfg=LEDC_AUTO_CLK};
    ESP_ERROR_CHECK(ledc_timer_config(&timer));
    const ledc_channel_config_t channel = {.gpio_num=22, .speed_mode=LEDC_LOW_SPEED_MODE,
        .channel=LEDC_CHANNEL_0, .timer_sel=LEDC_TIMER_0, .duty=0};
    ESP_ERROR_CHECK(ledc_channel_config(&channel));
    const spi_bus_config_t bus = {.sclk_io_num=7, .mosi_io_num=6, .miso_io_num=-1,
        .quadwp_io_num=-1, .quadhd_io_num=-1, .max_transfer_sz=172*40*2};
    ESP_ERROR_CHECK(spi_bus_initialize(SPI2_HOST, &bus, SPI_DMA_CH_AUTO));
    esp_lcd_panel_io_handle_t io;
    const esp_lcd_panel_io_spi_config_t io_config = {.dc_gpio_num=15, .cs_gpio_num=14,
        .pclk_hz=12000000, .lcd_cmd_bits=8, .lcd_param_bits=8, .spi_mode=0,
        .trans_queue_depth=10, .on_color_trans_done=done, .user_ctx=context};
    ESP_ERROR_CHECK(esp_lcd_new_panel_io_spi((esp_lcd_spi_bus_handle_t)SPI2_HOST, &io_config, &io));
    const esp_lcd_panel_dev_st7789t_config_t config = {.reset_gpio_num=21,
        .rgb_endian=LCD_RGB_ENDIAN_BGR, .bits_per_pixel=16};
    esp_lcd_panel_handle_t panel;
    ESP_ERROR_CHECK(esp_lcd_new_panel_st7789t(io, &config, &panel));
    ESP_ERROR_CHECK(esp_lcd_panel_reset(panel));
    ESP_ERROR_CHECK(esp_lcd_panel_init(panel));
    ESP_ERROR_CHECK(esp_lcd_panel_mirror(panel, true, false));
    ESP_ERROR_CHECK(esp_lcd_panel_set_gap(panel, 34, 0));
    ESP_ERROR_CHECK(esp_lcd_panel_disp_on_off(panel, true));
    const led_strip_config_t strip = {.strip_gpio_num=8, .max_leds=1};
    const led_strip_rmt_config_t rmt = {.resolution_hz=10000000, .flags.with_dma=false};
    ESP_ERROR_CHECK(led_strip_new_rmt_device(&strip, &rmt, &rgb));
    ESP_ERROR_CHECK(led_strip_clear(rgb));
    return panel;
}
