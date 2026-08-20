#ifndef VGPU_H
#define VGPU_H

#include <stdint.h>
#include <stdbool.h>

typedef struct vgpu_info {
    uint32_t width;
    uint32_t height;
    uint32_t bpp;
    uint32_t stride;
    uint64_t fb_phys;
    void *fb;
} vgpu_info_t;

bool vgpu_init(uint32_t w, uint32_t h);
void vgpu_shutdown(void);
vgpu_info_t vgpu_get_info(void);

void vgpu_clear(uint32_t color);
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color);
void vgpu_blit(const uint32_t *src, uint32_t x, uint32_t y,
               uint32_t w, uint32_t h);

#endif