#ifndef HDMI_INTERFACE_H
#define HDMI_INTERFACE_H

#include <stdint.h>
#include <stdbool.h>

#include "hdmi.h"

bool hdmi_iface_acquire(uint32_t owner);
bool hdmi_iface_release(uint32_t owner);
uint32_t hdmi_iface_owner(void);
bool hdmi_iface_check(uint32_t owner);

bool hdmi_iface_init(uint32_t owner, uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride);
bool hdmi_iface_ready(void);

bool hdmi_iface_mode_set(uint32_t owner, uint32_t id);
bool hdmi_iface_mode_current(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r);
bool hdmi_iface_mode_at(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r);

uint64_t hdmi_iface_submit_fill(uint32_t owner, uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
bool hdmi_iface_poll(uint32_t owner, uint64_t *out_seq);

void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);

bool hdmi_iface_fb_grant(uint32_t owner, uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *s);
bool hdmi_iface_fb_revoke(uint32_t owner);

#endif