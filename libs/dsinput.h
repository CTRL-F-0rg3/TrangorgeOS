#ifndef DSINPUT_H
#define DSINPUT_H
#include "dsabi.h"

static inline uint64_t ds_key_req(void)
{
    return ds_call(SVC_INPUT, IN_KEY_POLL, 0, 0, 0);
}

static inline int ds_key_take(uint64_t id, uint8_t *out)
{
    ds_msg_t m;

    if (!ds_take(id, &m) || m.arg0 == 0) {
        return 0;
    }

    *out = (uint8_t)m.arg0;
    return 1;
}

#endif