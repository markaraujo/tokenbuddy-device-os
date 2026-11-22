idf_component_register(SRCS "main.c" "../tests/test_behavior.c" INCLUDE_DIRS "."
    REQUIRES esp_system freertos log heap esp_timer board display avatar behavior)
