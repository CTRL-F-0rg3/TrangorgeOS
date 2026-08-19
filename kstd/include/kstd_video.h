#ifndef KSTD_VIDEO_H
#define KSTD_VIDEO_H

#include "kstd_types.h"

typedef struct tr_video_caps {
    uint64_t fb_phys;
    uint32_t width;
    uint32_t height;
    uint32_t stride;
    uint32_t ready;
} tr_video_caps_t;

typedef struct tr_video_mode {
    uint32_t id;
    uint32_t width;
    uint32_t height;
    uint32_t refresh;
} tr_video_mode_t;

/* Inicjalizacja i stan */
tr_status_t tr_video_init(void);
bool tr_video_is_ready(void);
tr_status_t tr_video_get_caps(tr_video_caps_t *caps);

/* Zarządzanie trybami */
uint32_t tr_video_mode_count(void);
tr_status_t tr_video_mode_get(uint32_t index, tr_video_mode_t *mode);
tr_status_t tr_video_mode_set(uint32_t mode_id);

/* Operacje na framebufferze */
tr_status_t tr_video_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
tr_status_t tr_video_blit(const void *src, uint32_t src_stride,
                          uint32_t dst_x, uint32_t dst_y, uint32_t w, uint32_t h);

#endif