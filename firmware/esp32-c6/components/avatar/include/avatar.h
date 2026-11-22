#pragma once
#include "behavior.h"
void avatar_init(void);
void avatar_update(tb_pose pose);
void avatar_visible(bool visible);
void avatar_context(bool online, bool demo);
