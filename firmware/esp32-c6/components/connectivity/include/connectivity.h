#pragma once
#include <stdbool.h>
void link_init(void);
bool link_read(char *line, unsigned capacity);
void link_write(const char *line);
