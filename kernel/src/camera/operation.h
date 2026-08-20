#ifndef CAMERA_OPERATION_H
#define CAMERA_OPERATION_H

#include "camera.h"

typedef struct camera_backend {
    bool (*present)(void);
    bool (*start)(void);
    bool (*stop)(void);
    bool (*frame)(void *buf, uint32_t cap, uint64_t *fid);
} camera_backend_t;

void camera_register_backend(const camera_backend_t *b);

#endif