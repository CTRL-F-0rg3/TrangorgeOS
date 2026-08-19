#include "kstd_fs.h"
#include "dsabi.h"

extern uint64_t ds_call(uint32_t cls, uint32_t op,
                        uint64_t a0, uint64_t a1, uint64_t a2);
extern void ds_poll(void);
extern int ds_take(uint64_t id, ds_msg_t *out);

static volatile uint8_t *scratch = (volatile uint8_t *)DS_SCRATCH_VA;

tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out)
{
    uint32_t n = 0;

    while (path[n] && n < 255) {
        scratch[n] = (uint8_t)path[n];
        n++;
    }

    scratch[n] = 0;

    uint64_t id = ds_call(SVC_FS, FS_READ, n, cap, 0);
    ds_poll();

    ds_msg_t r;

    if (!ds_take(id, &r) || r.status != 0) {
        return TR_ERR_NOTFOUND;
    }

    uint32_t got = (uint32_t)r.arg0;

    if (got > cap) {
        got = cap;
    }

    uint8_t *dst = (uint8_t *)buf;

    for (uint32_t i = 0; i < got; i++) {
        dst[i] = scratch[256 + i];
    }

    if (out) {
        *out = got;
    }

    return TR_OK;
}

tr_status_t tr_fs_exists(const char *path)
{
    uint32_t n = 0;

    while (path[n] && n < 255) {
        scratch[n] = (uint8_t)path[n];
        n++;
    }

    scratch[n] = 0;

    uint64_t id = ds_call(SVC_FS, FS_EXISTS, n, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0 && r.arg0 == 1) {
        return TR_OK;
    }

    return TR_ERR_NOTFOUND;
}