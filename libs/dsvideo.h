#ifndef DSVIDEO_H
#define DSVIDEO_H
#include "dsabi.h"

typedef struct {
    uint32_t w;
    uint32_t h;
    uint32_t stride;
    uint64_t phys;
} ds_fb_t;

static inline uint64_t ds_fb_info_req(void)
{
    return ds_call(SVC_VIDEO, VID_FB_INFO, 0, 0, 0);
}

static inline int ds_fb_info_take(uint64_t id, ds_fb_t *fb)
{
    ds_msg_t m;

    if (!ds_take(id, &m) || m.status != 0) {
        return 0;
    }

    fb->w = (uint32_t)(m.arg0 >> 16);
    fb->h = (uint32_t)(m.arg0 & 0xFFFF);
    fb->stride = (uint32_t)m.arg1;
    fb->phys = m.arg2;

    return 1;
}

static inline void ds_fb_takeover(void)
{
    ds_call(SVC_VIDEO, VID_FB_TAKEOVER, 0, 0, 0);
}

static inline void ds_fb_release(void)
{
    ds_call(SVC_VIDEO, VID_FB_RELEASE, 0, 0, 0);
}

static inline void ds_px(volatile uint32_t *fb, uint32_t stride,
                         uint32_t x, uint32_t y, uint32_t c)
{
    fb[(uint64_t)y * stride + x] = c;
}

static inline uint32_t ds_rgb(uint8_t r, uint8_t g, uint8_t b)
{
    return 0xFF000000u | ((uint32_t)r << 16) | ((uint32_t)g << 8) | b;
}

#endif