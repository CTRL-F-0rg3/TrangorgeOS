#include "hdmi.h"
#include "operation.h"

static bool hdmi_is_ready = false;

bool hdmi_init_with(uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride)
{
    if (hdmi_is_ready) {
        return true;
    }

    hdmi_op_set_fb(fb_phys, w, h, stride);

    hdmi_is_ready = true;

    return true;
}

bool hdmi_ready(void)
{
    return hdmi_is_ready;
}

void hdmi_caps(hdmi_caps_t *out)
{
    hdmi_op_state(out);
}

bool hdmi_mode_set(const hdmi_mode_t *m)
{
    if (m == NULL || m->w == 0 || m->h == 0) {
        return false;
    }

    return true;
}

bool hdmi_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
{
    hdmi_op_set_fb(phys, w, h, stride);
    return true;
}