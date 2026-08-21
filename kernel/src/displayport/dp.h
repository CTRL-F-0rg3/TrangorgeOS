#ifndef DP_H
#define DP_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "../hdmi/mode.h"

bool dp_init(void);
bool dp_ready(void);

void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);

uint32_t dp_mode_count(void);
bool dp_mode_at(uint32_t i, uint32_t *id, uint32_t *w,
                uint32_t *h, uint32_t *r);
bool dp_mode_set_by_id(uint32_t id);

void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride);
bool dp_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);

uint64_t dp_submit_fill(uint32_t color, uint32_t x, uint32_t y,
                        uint32_t w, uint32_t h);

#endif
