#include "kstd_mem.h"
#include "dsabi.h"

extern uint64_t ds_call(uint32_t cls, uint32_t op,
                        uint64_t a0, uint64_t a1, uint64_t a2);
extern void ds_poll(void);
extern int ds_take(uint64_t id, ds_msg_t *out);

static uint64_t next_va = 0x44000000ULL;

void *tr_alloc(uint32_t bytes)
{
    uint32_t pages = (bytes + 4095) / 4096;

    uint64_t id = ds_call(SVC_SYS, OP_ALLOC, pages, 0, 0);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        return (void *)r.arg0;
    }

    return TR_NULL;
}

void tr_free(void *ptr, uint32_t bytes)
{
    (void)bytes;
    ds_call(SVC_SYS, OP_FREE, (uint64_t)ptr, 0, 0);
}

tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va)
{
    uint64_t va = next_va;
    next_va += (len + 4095) & ~4095ULL;

    uint64_t id = ds_call(SVC_SYS, OP_MAPMMIO, phys, len, va);
    ds_poll();

    ds_msg_t r;

    if (ds_take(id, &r) && r.status == 0) {
        *out_va = (void *)va;
        return TR_OK;
    }

    return TR_ERR_DENIED;
}