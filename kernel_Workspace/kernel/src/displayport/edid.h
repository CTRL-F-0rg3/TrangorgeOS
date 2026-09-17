#ifndef DP_EDID_H
#define DP_EDID_H

#include <stdint.h>
#include <stdbool.h>

#include "../hdmi/mode.h"

bool edid_parse(const uint8_t *b, hdmi_mode_t *out,
                uint32_t max, uint32_t *count);

#endif
