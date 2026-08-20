#ifndef DSVGPU_H
#define DSVGPU_H

#include "dsabi.h"

extern uint64_t ds_call(uint32_t, uint32_t, uint64_t, uint64_t, uint64_t);
extern void ds_poll(void);
extern int ds_take(uint64_t, ds_msg_t *);

static inline int dsvgpu_surface_create(uint32_t w, uint32_t h,
                                        uint64_t *phys, int32_t *sid)
{
    uint64_t id = ds_call(SVC_VGPU, VGPU_SURF_CREATE, w, h, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return 0;
    }

    *sid = (int32_t)r.arg0;
    *phys = r.arg1;
    return 1;
}

static inline int dsvgpu_present(int32_t sid, uint32_t x, uint32_t y)
{
    uint64_t id = ds_call(SVC_VGPU, VGPU_PRESENT,
                          (uint64_t)(uint32_t)sid, x, y);
    ds_poll();

    ds_msg_t r;
    return ds_take(id, &r) && r.status == 0;
}

#endif