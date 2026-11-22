#pragma once
#include "esp_lcd_panel_io.h"
#include "esp_lcd_panel_ops.h"
esp_lcd_panel_handle_t board_init(esp_lcd_panel_io_color_trans_done_cb_t done, void *context);
void board_brightness(unsigned percent);
void board_rgb(unsigned red, unsigned green, unsigned blue);
bool board_boot_pressed(void);
