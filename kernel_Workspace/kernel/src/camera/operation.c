#include "operation.h"

extern void *arch_phys_to_virt(uint64_t phys);

#define CAM_W 320
#define CAM_H 240
#define CAM_BPP 3

static uint8_t framebuf[CAM_W * CAM_H * CAM_BPP];

static bool model_present_flag = true;
static bool streaming = false;
static uint64_t frame_id = 0;

static bool model_present(void)
{
    return model_present_flag;
}

static bool model_start(void)
{
    streaming = true;
    frame_id = 0;
    return true;
}

static bool model_stop(void)
{
    streaming = false;
    return true;
}

static void gen_frame(void)
{
    uint32_t f = (uint32_t)frame_id;

    for (uint32_t y = 0; y < CAM_H; y++) {
        for (uint32_t x = 0; x < CAM_W; x++) {
            uint8_t r = (uint8_t)(x + f * 4);
            uint8_t g = (uint8_t)y;
            uint8_t b = (uint8_t)(f * 2);

            if (((x + y + f * 8) & 0xFF) < 16) {
                r = g = b = 255;
            }

            uint32_t i = (y * CAM_W + x) * CAM_BPP;

            framebuf[i + 0] = r;
            framebuf[i + 1] = g;
            framebuf[i + 2] = b;
        }
    }
}

static bool model_frame(void *buf, uint32_t cap, uint64_t *fid)
{
    if (!streaming) {
        return false;
    }

    gen_frame();

    uint32_t n = sizeof(framebuf);

    if (cap < n) {
        n = cap;
    }

    uint8_t *dst = (uint8_t *)buf;

    for (uint32_t i = 0; i < n; i++) {
        dst[i] = framebuf[i];
    }

    frame_id++;

    if (fid) {
        *fid = frame_id;
    }

    return true;
}

static const camera_backend_t model_backend = {
    .present = model_present,
    .start = model_start,
    .stop = model_stop,
    .frame = model_frame,
};

static const camera_backend_t *backend = &model_backend;

void camera_register_backend(const camera_backend_t *b)
{
    if (b != (void *)0 && b->frame != (void *)0) {
        backend = b;
    }
}

bool camera_present(void)
{
    return backend->present();
}

bool camera_caps_get(camera_caps_t *out)
{
    if (out == (void *)0 || !camera_present()) {
        return false;
    }

    out->present = 1;
    out->width = CAM_W;
    out->height = CAM_H;
    out->format = CAM_FMT_RGB888;
    out->fps = 30;

    return true;
}

bool camera_start(void)
{
    if (!camera_present()) {
        return false;
    }

    return backend->start();
}

bool camera_stop(void)
{
    return backend->stop();
}

bool camera_frame(void *buf, uint32_t cap, uint64_t *fid)
{
    if (buf == (void *)0) {
        return false;
    }

    return backend->frame(buf, cap, fid);
}

bool camera_frame_to_phys(uint64_t phys, uint32_t cap, uint64_t *fid)
{
    return backend->frame(arch_phys_to_virt(phys), cap, fid);
}
