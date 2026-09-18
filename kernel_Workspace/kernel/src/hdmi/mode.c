#include "mode.h"
#include "operation.h"

static const hdmi_mode_t MODES[] = {
    { 1, 640, 480, 60, 25175, 16, 96, 160, 10, 2, 45, 0 },
    { 2, 800, 600, 60, 40000, 40, 128, 256, 1, 4, 28, 0 },
    { 3, 1024, 768, 60, 65000, 24, 136, 320, 3, 6, 38, 0 },
    { 4, 1280, 720, 60, 74250, 110, 40, 370, 5, 5, 25, 0 },
    { 5, 1280, 800, 60, 83500, 48, 32, 208, 3, 6, 28, 0 },
    { 6, 1920, 1080, 60, 148500, 88, 44, 280, 4, 5, 45, 0 },
};

static const hdmi_mode_t *current_mode = &MODES[0];

uint32_t hdmi_mode_count(void)
{
    return (uint32_t)(sizeof(MODES) / sizeof(MODES[0]));
}

const hdmi_mode_t *hdmi_mode_at(uint32_t i)
{
    if (i >= hdmi_mode_count()) {
        return NULL;
    }

    return &MODES[i];
}

const hdmi_mode_t *hdmi_mode_find(uint32_t w, uint32_t h, uint32_t refresh)
{
    for (uint32_t i = 0; i < hdmi_mode_count(); i++) {
        if (MODES[i].w == w && MODES[i].h == h && MODES[i].refresh == refresh) {
            return &MODES[i];
        }
    }

    return NULL;
}

const hdmi_mode_t *hdmi_mode_by_id(uint32_t id)
{
    for (uint32_t i = 0; i < hdmi_mode_count(); i++) {
        if (MODES[i].id == id) {
            return &MODES[i];
        }
    }

    return NULL;
}

bool hdmi_mode_valid(const hdmi_mode_t *m)
{
    if (m == NULL || m->w == 0 || m->h == 0 || m->refresh == 0 || m->clock_khz == 0) {
        return false;
    }

    hdmi_caps_t caps;
    hdmi_op_state(&caps);
    return caps.ready != 0 && caps.fb_phys != 0;
}

bool hdmi_mode_apply(const hdmi_mode_t *m)
{
    if (!hdmi_mode_valid(m)) {
        return false;
    }

    hdmi_op_set_mode(m);
    current_mode = m;
    return true;
}

const hdmi_mode_t *hdmi_mode_current(void)
{
    return current_mode;
}

bool hdmi_mode_at_raw(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
{
    const hdmi_mode_t *m = hdmi_mode_at(i);
    if (m == NULL || id == NULL || w == NULL || h == NULL || r == NULL) {
        return false;
    }

    *id = m->id;
    *w = m->w;
    *h = m->h;
    *r = m->refresh;
    return true;
}
