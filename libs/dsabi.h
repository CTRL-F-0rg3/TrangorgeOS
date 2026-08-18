#ifndef DSABI_H
#define DSABI_H

#include <stdint.h>

#define DS_MAGIC 0x4452565350414345ULL
#define DS_RING_CAP 16

#define SVC_SYS 0
#define SVC_VIDEO 1
#define SVC_AUDIO 2
#define SVC_INPUT 3
#define SVC_BLOCK 4

#define svc_cmd(cls, op) (((cls) << 8) | ((op) & 0xFF))

typedef struct {
    uint64_t id;
    uint32_t cmd;
    uint32_t flags;
    uint64_t arg0;
    uint64_t arg1;
    uint64_t arg2;
    int32_t status;
    uint32_t pad;
} ds_msg_t;

typedef struct {
    uint64_t head;
    uint64_t tail;
    uint64_t cap;
} ds_ring_t;

typedef struct {
    uint64_t magic;
    uint32_t version;
    uint32_t pad;
    uint64_t k2d_va;
    uint64_t d2k_va;
    uint64_t ring_cap;
    uint64_t ds_va_base;
    uint64_t ds_va_size;
} ds_params_t;

void ds_init(uint64_t params_va);
uint64_t ds_call(uint32_t cls, uint32_t op,
                 uint64_t a0, uint64_t a1, uint64_t a2);
int ds_take(uint64_t id, ds_msg_t *out);
void ds_poll(void);

#endif