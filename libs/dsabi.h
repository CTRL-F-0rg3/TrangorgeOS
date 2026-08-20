#ifndef DSABI_H
#define DSABI_H
#define SVC_FS 8

#define FS_READ   1
#define FS_EXISTS 2
#include <stdint.h>

#define DS_MAGIC 0x4452565350414345ULL
#define DS_RING_CAP 16

#define DS_INIT_PARAMS_VA 0x40000000ULL
#define DS_K2D_VA 0x40001000ULL
#define DS_D2K_VA 0x40002000ULL
#define DS_SCRATCH_VA 0x40003000ULL
#define DS_SCRATCH_SIZE 4096

#define SVC_SYS   0
#define SVC_VIDEO 1
#define SVC_AUDIO 2
#define SVC_INPUT 3
#define SVC_BLOCK 4
#define SVC_NET   5

#define svc_cmd(cls, op) ((((uint32_t)(cls)) << 8) | ((op) & 0xFF))

/* SYS ops */
#define OP_LOG        10
#define OP_ALLOC      11
#define OP_FREE       12
#define OP_MAPMMIO    13
#define OP_DEVCOUNT   14
#define OP_BLKREAD    15
#define OP_BLKWRITE   16
#define OP_AUDIOINFO  48
#define OP_PAGEPHYS   49

/* VIDEO ops */
#define VID_FB_INFO     1
#define VID_FB_TAKEOVER 2
#define VID_FB_RELEASE  3

/* INPUT ops */
#define IN_KEY_POLL 1

/* AUDIO ops */
#define AUD_PLAY 1
#define AUD_STOP 2
#define AUD_JACK 3
#define AUD_AMP  4

/* BLOCK ops */
#define BLK_COUNT 1
#define BLK_READ  2
#define BLK_WRITE 3

#define SVC_PCI 11

#define PCI_FIND   1
#define PCI_BAR    2
#define PCI_ENABLE 3
#define PORT_WRITE 4
#define PORT_READ  5

#define OP_REG_DRIVER 17

#define DRIVER_KIND_VIDEO 1
#define DRIVER_KIND_AUDIO 2
#define DRIVER_KIND_NET   3

static inline void ds_yield(void)
{
    __asm__ volatile("int $0x80" :: "a"(1));
}

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