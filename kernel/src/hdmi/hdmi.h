#ifndef HDMI_H
#define HDMI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "mode.h"

#define HDMI_OP_FB_SET   1
#define HDMI_OP_MODE_SET 2
#define HDMI_OP_TRANSFER 3
#define HDMI_OP_POLL     4

#define HDMI_TR_BLIT 1
#define HDMI_TR_FILL 2
#define HDMI_TR_FLIP 3

#define HDMI_MAX_TRANSFERS 16

typedef struct hdmi_transfer {
    uint32_t kind;
    uint32_t color;
    uint64_t src_phys;
    uint64_t dst_phys;
    uint32_t x, y, w, h;
    uint32_t stride;
} hdmi_transfer_t;

typedef struct hdmi_caps {
    uint64_t fb_phys;
    uint32_t w, h, stride;
    uint32_t ready;
} hdmi_caps_t;

bool hdmi_init_with(uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride);
bool hdmi_ready(void);
void hdmi_caps(hdmi_caps_t *out);
bool hdmi_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
bool hdmi_mode_set_by_id(uint32_t id);

uint64_t hdmi_submit(const hdmi_transfer_t *t);
bool hdmi_poll(uint64_t *out_seq);
uint32_t hdmi_pending(void);

uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y,
                          uint32_t w, uint32_t h);
void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);

#endif