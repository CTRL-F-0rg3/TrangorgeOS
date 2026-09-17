#ifndef HDMI_MODE_H
#define HDMI_MODE_H

#include <stdint.h>
#include <stdbool.h>

#define HDMI_MODE_FLAG_INTERLACE (1u << 0)
#define HDMI_MODE_FLAG_PVSYNC    (1u << 1)
#define HDMI_MODE_FLAG_PHSYNC    (1u << 2)

typedef struct hdmi_mode {
    uint32_t id;
    uint32_t w, h, refresh;

    uint32_t clock_khz;

    uint32_t h_front, h_sync, h_blank;
    uint32_t v_front, v_sync, v_blank;

    uint32_t flags;
} hdmi_mode_t;

uint32_t hdmi_mode_count(void);
const hdmi_mode_t *hdmi_mode_at(uint32_t i);
const hdmi_mode_t *hdmi_mode_find(uint32_t w, uint32_t h, uint32_t refresh);
const hdmi_mode_t *hdmi_mode_by_id(uint32_t id);
bool hdmi_mode_valid(const hdmi_mode_t *m);
bool hdmi_mode_apply(const hdmi_mode_t *m);
const hdmi_mode_t *hdmi_mode_current(void);
bool hdmi_mode_at_raw(uint32_t i, uint32_t *id, uint32_t *w,
                      uint32_t *h, uint32_t *r);

#endif