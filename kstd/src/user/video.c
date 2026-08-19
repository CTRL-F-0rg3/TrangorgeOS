#include "kstd_video.h"
#include "dsabi.h"      // Z Twojego libs/ (SVC_VIDEO, VID_HDMI_FILL itd.)

/* Zakładamy, że ds_call i ds_take są dostępne z dsclient.c */

tr_status_t tr_video_init(void) {
    /* W user-space init robi kernel, tu tylko sprawdzamy stan */
    return tr_video_is_ready() ? TR_OK : TR_ERR_IO;
}

bool tr_video_is_ready(void) {
    uint64_t id = ds_call(SVC_VIDEO, VID_HDMI_CAPS, 0, 0, 0);
    ds_poll();
    ds_msg_t r;
    if (ds_take(id, &r) && r.status == 0) {
        return true;
    }
    return false;
}

tr_status_t tr_video_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h) {
    uint64_t arg1 = ((uint64_t)x) | ((uint64_t)y << 16);
    uint64_t arg2 = ((uint64_t)w) | ((uint64_t)h << 16);
    
    uint64_t id = ds_call(SVC_VIDEO, VID_HDMI_FILL, color, arg1, arg2);
    ds_poll();
    
    ds_msg_t r;
    if (ds_take(id, &r)) {
        if (r.status == 0) return TR_OK;
        if (r.status == -1) return TR_ERR_DENIED; // Autoryzacja/Budżet
        return TR_ERR_IO;
    }
    return TR_ERR_TIMEOUT;
}

/* ... reszta funkcji serializuje argumenty do DsMsg ... */