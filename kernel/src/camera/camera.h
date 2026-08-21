#ifndef CAMERA_H
#define CAMERA_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define CAM_FMT_RGB888 1
#define CAM_FMT_YUYV   2

typedef struct camera_caps {
    uint32_t present;
    uint32_t width;
    uint32_t height;
    uint32_t format;
    uint32_t fps;
} camera_caps_t;

bool camera_init(void);
bool camera_present(void);
bool camera_caps_get(camera_caps_t *out);
bool camera_start(void);
bool camera_stop(void);
bool camera_frame(void *buf, uint32_t cap, uint64_t *frame_id);
bool camera_frame_to_phys(uint64_t phys, uint32_t cap, uint64_t *frame_id);

#endif
