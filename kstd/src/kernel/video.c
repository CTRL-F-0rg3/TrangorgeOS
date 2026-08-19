#include "kstd_video.h"
#include "/home/ctrl/TrangorgeOS/kernel/src/hdmi/hdmi.h"
#include "/home/ctrl/TrangorgeOS/kernel/src/displayport/dp.h"

/* W kernelu wybieramy aktywny backend (np. ten, który się pierwszy zainicjował) */
static bool use_dp = false;

tr_status_t tr_video_init(void) {
    if (dp_ready()) {
        use_dp = true;
        return TR_OK;
    }
    if (hdmi_ready()) {
        use_dp = false;
        return TR_OK;
    }
    return TR_ERR_IO;
}

bool tr_video_is_ready(void) {
    return use_dp ? dp_ready() : hdmi_ready();
}

tr_status_t tr_video_get_caps(tr_video_caps_t *caps) {
    if (!caps) return TR_ERR_INVALID;

    if (use_dp) {
        dp_caps(&caps->fb_phys, &caps->width, &caps->height, &caps->stride);
        caps->ready = dp_ready();
    } else {
        hdmi_caps_raw(&caps->width, &caps->height, &caps->stride, &caps->fb_phys);
        caps->ready = hdmi_ready();
    }
    return TR_OK;
}

tr_status_t tr_video_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h) {
    if (!tr_video_is_ready()) return TR_ERR_IO;

    uint64_t seq = 0;
    if (use_dp) {
        seq = dp_submit_fill(color, x, y, w, h);
    } else {
        seq = hdmi_submit_fill(color, x, y, w, h);
    }

    return (seq > 0) ? TR_OK : TR_ERR_BUSY;
}

/* ... reszta funkcji mapuje 1:1 na hdmi/dp ... */