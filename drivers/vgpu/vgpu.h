#ifndef VGPU_H
#define VGPU_H

#include <stdint.h>
#include <stdbool.h>

#define VGPU_MAX_SURFACES 8

typedef struct vgpu_info {
    uint32_t width, height, bpp, stride;
    uint64_t fb_phys;
    void *fb;
} vgpu_info_t;

typedef struct vgpu_surface {
    uint64_t phys;
    void *va;
    uint32_t w, h;
    bool used;
} vgpu_surface_t;

bool vgpu_init(uint32_t w, uint32_t h);
vgpu_info_t vgpu_get_info(void);

void vgpu_clear(uint32_t color);
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color);

int32_t vgpu_surface_create(uint32_t w, uint32_t h, uint64_t *phys_out);
bool vgpu_surface_present(int32_t id, uint32_t x, uint32_t y);

void vgpu_process_ring(volatile void *ring);

#endif