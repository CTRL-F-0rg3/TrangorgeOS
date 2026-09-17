#include "dp.h"
#include "aux.h"
#include "link.h"
#include "edid.h"
#include "operation.h"

static hdmi_mode_t MODES[8];
static uint32_t MODE_COUNT = 0;
static bool dp_is_ready = false;

bool dp_init(void)
{
    if (dp_is_ready) {
        return true;
    }

    dp_aux_sink_init();

    if (!dp_link_train()) {
        return false;
    }

    uint8_t raw[128];

    if (!dp_aux_read_edid(raw, 128)) {
        return false;
    }

    if (!edid_parse(raw, MODES, 8, &MODE_COUNT)) {
        return false;
    }

    dp_is_ready = true;

    return true;
}

bool dp_ready(void)
{
    return dp_is_ready;
}

uint32_t dp_mode_count(void)
{
    return MODE_COUNT;
}

bool dp_mode_at(uint32_t i, uint32_t *id, uint32_t *w,
                uint32_t *h, uint32_t *r)
{
    if (i >= MODE_COUNT) {
        return false;
    }

    *id = MODES[i].id;
    *w = MODES[i].w;
    *h = MODES[i].h;
    *r = MODES[i].refresh;

    return true;
}

bool dp_mode_set_by_id(uint32_t id)
{
    for (uint32_t i = 0; i < MODE_COUNT; i++) {
        if (MODES[i].id == id) {
            dp_op_set_fb(0, MODES[i].w, MODES[i].h, MODES[i].w);
            return true;
        }
    }

    return false;
}

void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride)
{
    dp_op_state(fb_phys, w, h, stride);
}

bool dp_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
{
    dp_op_set_fb(phys, w, h, stride);
    return true;
}

uint64_t dp_submit_fill(uint32_t color, uint32_t x, uint32_t y,
                        uint32_t w, uint32_t h)
{
    static uint64_t seq = 0;

    dp_op_fill(color, x, y, w, h);

    return ++seq;
}
