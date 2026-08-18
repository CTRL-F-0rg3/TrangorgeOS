#ifndef DSASYNC_H
#define DSASYNC_H

#include "dsabi.h"

typedef struct {
    uint64_t id;
    int done;
    ds_msg_t msg;
} ds_req_t;

static inline void ds_req_begin(ds_req_t *r, uint32_t cls, uint32_t op,
                                uint64_t a0, uint64_t a1, uint64_t a2)
{
    r->id = ds_call(cls, op, a0, a1, a2);
    r->done = 0;
}

static inline int ds_req_poll(ds_req_t *r)
{
    if (r->done) {
        return 1;
    }

    ds_poll();

    if (r->id != 0 && ds_take(r->id, &r->msg)) {
        r->done = 1;
    }

    return r->done;
}

static inline int ds_req_ok(ds_req_t *r)
{
    return r->done && r->msg.status == 0;
}

#endif

static ds_req_t req;

// void tick(void) {
//     if (!req.done) {
//         if (ds_req_poll(&req)) { /* odpowiedź */ }
//         return;
//     }
//     ds_req_begin(&req, SVC_VIDEO, VID_FB_INFO, 0, 0, 0);
// }