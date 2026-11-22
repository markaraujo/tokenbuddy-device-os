idf_component_register(SRCS "main.c" "../tests/test_behavior.c" "../tests/test_control.c" INCLUDE_DIRS "."
    REQUIRES esp_system freertos log heap esp_timer board display avatar behavior control ui connectivity nvs_flash esp_hw_support)
