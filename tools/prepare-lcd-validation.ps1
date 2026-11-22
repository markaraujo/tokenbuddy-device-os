# Prepara fontes apenas. Nunca grava firmware.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$vendor = Join-Path $root '.tools/waveshare-demo/ESP-IDF/ESP32-C6-LCD-1.47-Test'
$testProject = Join-Path $root '.tools/lcd-validation'
if (Test-Path $testProject) { throw 'Projeto de teste existente; preservar e usar idf.py build nele.' }
if (-not (Test-Path "$vendor/main/LCD_Driver/ST7789.c")) { throw 'Demo oficial ausente.' }
New-Item -ItemType Directory -Force "$testProject/main", "$testProject/components" | Out-Null
Copy-Item "$vendor/main/LCD_Driver", "$vendor/main/LVGL_Driver" "$testProject/main" -Recurse
Copy-Item "$vendor/components/lvgl__lvgl" "$testProject/components" -Recurse
Copy-Item "$PSScriptRoot/lcd-validation/main.c" "$testProject/main/main.c"
# O demo aplica 75% durante LCD_Init. Limitar desde a primeira ativacao.
$driverPath = "$testProject/main/LCD_Driver/ST7789.c"
$driver = Get-Content -Raw $driverPath
if (-not $driver.Contains('BK_Light(75);')) { throw 'Driver divergente; revisar antes de adaptar.' }
$driver.Replace('BK_Light(75);', 'BK_Light(30);') | Set-Content -Encoding UTF8 $driverPath
@'
cmake_minimum_required(VERSION 3.16)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(tokenbuddy_lcd_validation)
'@ | Set-Content -Encoding UTF8 "$testProject/CMakeLists.txt"
@'
idf_component_register(SRCS "main.c" "LCD_Driver/ST7789.c" "LCD_Driver/Vernon_ST7789T/Vernon_ST7789T.c" "LVGL_Driver/LVGL_Driver.c"
    INCLUDE_DIRS "." "LCD_Driver" "LCD_Driver/Vernon_ST7789T" "LVGL_Driver"
    REQUIRES esp_lcd driver esp_timer lvgl__lvgl)
'@ | Set-Content -Encoding UTF8 "$testProject/main/CMakeLists.txt"
@'
CONFIG_IDF_TARGET="esp32c6"
CONFIG_ESPTOOLPY_FLASHSIZE_4MB=y
CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG=y
'@ | Set-Content -Encoding UTF8 "$testProject/sdkconfig.defaults"
Write-Output "Preparado em $testProject. Nenhum flash executado."
