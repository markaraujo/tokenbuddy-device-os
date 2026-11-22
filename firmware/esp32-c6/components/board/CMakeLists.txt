idf_component_register(SRCS "board.c" "vendor/Vernon_ST7789T/Vernon_ST7789T.c"
    INCLUDE_DIRS "include" PRIV_INCLUDE_DIRS "vendor"
    REQUIRES esp_lcd driver led_strip)
