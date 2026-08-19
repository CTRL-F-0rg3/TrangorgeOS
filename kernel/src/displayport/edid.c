#include "edid.h"

bool edid_parse(const uint8_t *b, hdmi_mode_t *out,
                uint32_t max, uint32_t *count)
{
    *count = 0;

    if (b[0] != 0x00 || b[1] != 0xFF || b[7] != 0x00) {
        return false;
    }

    for (uint32_t d = 0; d < 4; d++) {
        const uint8_t *p = &b[54 + d * 18];

        uint32_t clock = (uint32_t)p[0] | ((uint32_t)p[1] << 8);

        if (clock == 0) {
            continue;
        }

        uint32_t hact = p[2] | (((uint32_t)p[4] >> 4) << 8);
        uint32_t hblk = p[3] | (((uint32_t)p[4] & 0xF) << 8);
        uint32_t vact = p[5] | (((uint32_t)p[7] >> 4) << 8);
        uint32_t vblk = p[6] | (((uint32_t)p[7] & 0xF) << 8);

        uint32_t hsync = p[9] | ((((uint32_t)p[10] >> 4) & 0xF) << 8);
        uint32_t vsync = (uint32_t)p[10] & 0xF;
        uint32_t hoff = p[8] | (((uint32_t)p[11] >> 6) << 8);
        uint32_t voff = ((uint32_t)p[11] & 0x30) >> 4
                      | ((((uint32_t)p[10] >> 2) & 0x0C) << 2);

        if (hact == 0 || vact == 0) {
            continue;
        }

        if (*count >= max) {
            break;
        }

        uint32_t htot = hact + hblk;
        uint32_t vtot = vact + vblk;

        uint32_t refresh = (clock * 1000u) / (htot * vtot);

        hdmi_mode_t *m = &out[*count];

        m->id = 100 + *count;
        m->w = hact;
        m->h = vact;
        m->refresh = refresh;
        m->clock_khz = clock * 10;
        m->h_front = hoff;
        m->h_sync = hsync;
        m->h_blank = hblk;
        m->v_front = voff;
        m->v_sync = vsync;
        m->v_blank = vblk;
        m->flags = 0;

        (*count)++;
    }

    return *count != 0;
}