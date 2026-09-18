#include "hdmi.h"
#include "operation.h"

static bool hdmi_is_ready;

bool hdmi_init_with(uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride)
{
    if (hdmi_is_ready) {
        return true;
    }

    if (fb_phys == 0 || w == 0 || h == 0 || stride < w) {
        return false;
    }

    hdmi_op_set_fb(fb_phys, w, h, stride);
    if (!hdmi_op_ready()) {
        return false;
    }

    const hdmi_mode_t *mode = hdmi_mode_find(w, h, 60);
    if (mode != NULL && !hdmi_mode_apply(mode)) {
        return false;
    }

    hdmi_is_ready = true;
    return true;
}

bool hdmi_ready(void)
{
    return hdmi_is_ready && hdmi_op_ready();
}

void hdmi_caps(hdmi_caps_t *out)
{
    hdmi_op_state(out);
}

bool hdmi_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
{
    if (phys == 0 || w == 0 || h == 0 || stride < w) {
        return false;
    }

    hdmi_op_set_fb(phys, w, h, stride);
    hdmi_is_ready = hdmi_op_ready();
    return hdmi_is_ready;
}

bool hdmi_mode_set_by_id(uint32_t id)
{
    if (!hdmi_ready()) {
        return false;
    }

    const hdmi_mode_t *mode = hdmi_mode_by_id(id);
    hdmi_caps_t caps;
    hdmi_op_state(&caps);
    if (mode == NULL || mode->w != caps.w || mode->h != caps.h) {
        return false;
    }

    return hdmi_mode_apply(mode);
}

bool hdmi_mode_current_raw(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
{
    const hdmi_mode_t *mode = hdmi_mode_current();
    if (mode == NULL || id == NULL || w == NULL || h == NULL || r == NULL) {
        return false;
    }

    *id = mode->id;
    *w = mode->w;
    *h = mode->h;
    *r = mode->refresh;
    return true;
}
