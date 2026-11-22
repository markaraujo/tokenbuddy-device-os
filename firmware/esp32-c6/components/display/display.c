#include "display.h"
#include "board.h"
#include "lvgl.h"
#include "esp_timer.h"

static lv_color_t buffers[2][172*20];
static lv_disp_draw_buf_t draw_buffer;
static lv_disp_drv_t driver;
static esp_lcd_panel_handle_t panel;

static bool transfer_done(esp_lcd_panel_io_handle_t io, esp_lcd_panel_io_event_data_t *event, void *context)
{
    (void)io; (void)event;
    lv_disp_flush_ready(context);
    return false;
}
static void flush(lv_disp_drv_t *drv, const lv_area_t *area, lv_color_t *colors)
{
    (void)drv;
    ESP_ERROR_CHECK(esp_lcd_panel_draw_bitmap(panel, area->x1, area->y1,
        area->x2+1, area->y2+1, colors));
}
static void tick(void *context) { (void)context; lv_tick_inc(2); }

void display_init(void)
{
    lv_init();
    lv_disp_drv_init(&driver);
    panel = board_init(transfer_done, &driver);
    lv_disp_draw_buf_init(&draw_buffer, buffers[0], buffers[1], 172*20);
    driver.hor_res=172; driver.ver_res=320;
    driver.flush_cb=flush; driver.draw_buf=&draw_buffer;
    lv_disp_drv_register(&driver);
    const esp_timer_create_args_t args={.callback=tick, .name="lv_tick"};
    esp_timer_handle_t timer;
    ESP_ERROR_CHECK(esp_timer_create(&args, &timer));
    ESP_ERROR_CHECK(esp_timer_start_periodic(timer, 2000));
}
void display_process(void) { lv_timer_handler(); }
