#include "operation.h"

extern void *arch_phys_to_virt(uint64_t phys);

static uint64_t fb_phys = 0;
static uint32_t w = 0, h = 0, stride = 0;

void dp_op_set_fb(uint64_t phys, uint32_t ww, uint32_t hh, uint32_t s)
{
    fb_phys = phys;
    w = ww;
    h = hh;
    stride = s;
}

void dp_op_state(uint64_t *phys, uint32_t *ww, uint32_t *hh, uint32_t *s)
{
    *phys = fb_phys;
    *ww = w;
    *hh = h;
    *s = stride;
}

void dp_op_fill(uint32_t color, uint32_t x, uint32_t y,
                uint32_t fw, uint32_t fh)
{
    if (fb_phys == 0) {
        return;
    }

    uint32_t *dst = (uint32_t *)arch_phys_to_virt(fb_phys);

    for (uint32_t yy = y; yy < y + fh && yy < h; yy++) {
        for (uint32_t xx = x; xx < x + fw && xx < w; xx++) {
            dst[yy * stride + xx] = color;
        }
    }
}