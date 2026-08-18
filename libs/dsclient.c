#include "dsabi.h"

static volatile ds_ring_t *k2d;
static volatile ds_ring_t *d2k;
static volatile ds_msg_t *k2d_msgs;
static volatile ds_msg_t *d2k_msgs;
static uint64_t next_id = 5000;

static ds_msg_t cache[16];
static uint8_t cache_used[16];

void ds_init(uint64_t params_va)
{
    volatile ds_params_t *p = (volatile ds_params_t *)params_va;

    k2d = (volatile ds_ring_t *)p->k2d_va;
    d2k = (volatile ds_ring_t *)p->d2k_va;
    k2d_msgs = (volatile ds_msg_t *)(p->k2d_va + sizeof(ds_ring_t));
    d2k_msgs = (volatile ds_msg_t *)(p->d2k_va + sizeof(ds_ring_t));
}

uint64_t ds_call(uint32_t class, uint32_t op,
                 uint64_t a0, uint64_t a1, uint64_t a2)
{
    uint64_t id = next_id++;

    uint64_t head = d2k->head;
    uint64_t next = (head + 1) % d2k->cap;

    if (next == d2k->tail) {
        return 0;
    }

    volatile ds_msg_t *m = &d2k_msgs[head];

    m->id = id;
    m->cmd = svc_cmd(class, op);
    m->flags = 1;
    m->arg0 = a0;
    m->arg1 = a1;
    m->arg2 = a2;
    m->status = 0;

    d2k->head = next;

    return id;
}

void ds_poll(void)
{
    while (k2d->tail != k2d->head) {
        volatile ds_msg_t *m = &k2d_msgs[k2d->tail];
        ds_msg_t copy = *m;

        k2d->tail = (k2d->tail + 1) % k2d->cap;

        for (int i = 0; i < 16; i++) {
            if (!cache_used[i]) {
                cache[i] = copy;
                cache_used[i] = 1;
                break;
            }
        }
    }
}

int ds_take(uint64_t id, ds_msg_t *out)
{
    for (int i = 0; i < 16; i++) {
        if (cache_used[i] && cache[i].id == id) {
            *out = cache[i];
            cache_used[i] = 0;
            return 1;
        }
    }

    return 0;
}

// ds_init(params_va);
// uint64_t id = ds_call(SVC_VIDEO, VID_FB_INFO, 0, 0, 0);
// ds_poll();
// ds_msg_t r;
// if (ds_take(id, &r)) { /* r.arg2 = fb phys */ }