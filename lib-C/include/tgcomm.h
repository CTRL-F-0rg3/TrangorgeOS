/*
 * tgcomm.h — C bindings for the TrangorgeOS strict shared-memory
 *            communication protocol.
 *
 * This header mirrors the PUBLIC protocol (message layout, layers, opcodes,
 * rights, object types) of the Rust core in `../lib`. The capability table,
 * rings and the authorization gate remain opaque: C never re-implements them,
 * it only calls the stable `tgcomm_*` C ABI provided by `libtg_comm.a`.
 */
#ifndef TGCOMM_H
#define TGCOMM_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------ */
/* Protocol constants (mirror lib/src/consts.rs)                      */
/* ------------------------------------------------------------------ */
#define TGCOMM_MAGIC              0x5447434Du /* "TGCM" */
#define TGCOMM_VERSION            1u
#define TGCOMM_MSG_SIZE           64u
#define TGCOMM_RING_CTRL          16u
#define TGCOMM_RING_SLOTS_DEFAULT 256u
#define TGCOMM_CAP_TABLE_SIZE     512u
/* Opaque table storage reserved by the C side; >= tgcomm_cap_table_size(). */
#define TGCOMM_CAP_TABLE_BYTES    16384u

/* ------------------------------------------------------------------ */
/* Message kind                                                        */
/* ------------------------------------------------------------------ */
typedef enum tgcomm_msg_kind {
    TGCOMM_KIND_REQUEST      = 0,
    TGCOMM_KIND_REPLY        = 1,
    TGCOMM_KIND_EVENT        = 2,
    TGCOMM_KIND_NOTIFICATION = 3,
    TGCOMM_KIND_CAP_TRANSFER = 4,
} tgcomm_msg_kind_t;

/* ------------------------------------------------------------------ */
/* Layers                                                              */
/* ------------------------------------------------------------------ */
typedef enum tgcomm_layer {
    TGCOMM_LAYER_KERNEL           = 0,
    TGCOMM_LAYER_DRIVERSPACE      = 1,
    TGCOMM_LAYER_USERSPACE        = 2,
    TGCOMM_LAYER_MANAGER          = 3,
    TGCOMM_LAYER_USER_DRIVER_SPACE = 4,
} tgcomm_layer_t;

/* ------------------------------------------------------------------ */
/* Object types                                                        */
/* ------------------------------------------------------------------ */
typedef enum tgcomm_obj_type {
    TGCOMM_OBJ_MEMORY       = 0,
    TGCOMM_OBJ_ENDPOINT     = 1,
    TGCOMM_OBJ_NOTIFICATION = 2,
    TGCOMM_OBJ_CHANNEL      = 3,
    TGCOMM_OBJ_SHMEM_REGION = 4,
    TGCOMM_OBJ_DEVICE       = 5,
    TGCOMM_OBJ_IRQ          = 6,
} tgcomm_obj_type_t;

/* ------------------------------------------------------------------ */
/* Rights bitmask                                                      */
/* ------------------------------------------------------------------ */
#define TGCOMM_RIGHT_READ     (1u << 0)
#define TGCOMM_RIGHT_WRITE    (1u << 1)
#define TGCOMM_RIGHT_EXEC     (1u << 2)
#define TGCOMM_RIGHT_MAP      (1u << 3)
#define TGCOMM_RIGHT_GRANT    (1u << 4)
#define TGCOMM_RIGHT_TRANSFER (1u << 5)
#define TGCOMM_RIGHT_SEND     (1u << 6)
#define TGCOMM_RIGHT_RECV     (1u << 7)
#define TGCOMM_RIGHT_CALL     (1u << 8)
#define TGCOMM_RIGHT_MANAGE   (1u << 9)

/* ------------------------------------------------------------------ */
/* Opcode classes                                                      */
/* ------------------------------------------------------------------ */
#define TGCOMM_OP_SYS   0
#define TGCOMM_OP_MEM   1
#define TGCOMM_OP_CAP   2
#define TGCOMM_OP_IPC   3
#define TGCOMM_OP_SHMEM 4
#define TGCOMM_OP_VIDEO 5
#define TGCOMM_OP_AUDIO 6
#define TGCOMM_OP_INPUT 7
#define TGCOMM_OP_BLOCK 8
#define TGCOMM_OP_NET   9
#define TGCOMM_OP_PCI   10
#define TGCOMM_OP_VGPU  11
#define TGCOMM_OP_FS    12

#define TGCOMM_OPCODE(cls, op) (((uint32_t)(cls) << 8) | ((uint32_t)(op) & 0xFFu))

/* ------------------------------------------------------------------ */
/* The fixed-size wire message (mirror lib/src/wire.rs, 64 bytes)     */
/* ------------------------------------------------------------------ */
typedef struct tgcomm_msg {
    uint32_t magic;
    uint8_t  version;
    uint8_t  kind;
    uint8_t  layer;
    uint8_t  target;
    uint32_t opcode;
    uint32_t cap;
    uint32_t seq;
    int32_t  status;
    uint64_t a0;
    uint64_t a1;
    uint64_t a2;
    uint64_t a3;
    uint8_t  reserved[8];
} tgcomm_msg_t;

#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
_Static_assert(sizeof(tgcomm_msg_t) == TGCOMM_MSG_SIZE, "tgcomm_msg_t must be 64 bytes");
#endif

/* ------------------------------------------------------------------ */
/* Authorize result (mirror filter.rs)                                */
/* ------------------------------------------------------------------ */
typedef enum tgcomm_result {
    TGCOMM_FORWARDED           = 0,
    TGCOMM_MALFORMED           = 1,
    TGCOMM_UNKNOWN_OP          = 2,
    TGCOMM_ROUTE_DENIED        = 3,
    TGCOMM_NO_CAPABILITY       = 4,
    TGCOMM_RIGHTS_INSUFFICIENT = 5,
    TGCOMM_TYPE_MISMATCH       = 6,
} tgcomm_result_t;

/* ------------------------------------------------------------------ */
/* Opaque capability table (real layout lives in Rust)                */
/* ------------------------------------------------------------------ */
typedef struct tgcomm_cap_table {
    _Alignas(16) uint8_t storage[TGCOMM_CAP_TABLE_BYTES];
} tgcomm_cap_table_t;

/* ------------------------------------------------------------------ */
/* Stable C ABI (provided by libtg_comm.a)                            */
/* ------------------------------------------------------------------ */
uint32_t tgcomm_version(void);
uint32_t tgcomm_msg_size(void);
size_t   tgcomm_ring_size(uint32_t slots);
size_t   tgcomm_cap_table_size(void);

void     tgcomm_msg_init(tgcomm_msg_t *msg);
int32_t  tgcomm_msg_valid(const tgcomm_msg_t *msg);

int32_t  tgcomm_authorize(const tgcomm_cap_table_t *table,
                          const tgcomm_msg_t *msg);

void     tgcomm_ring_init(void *ring, uint32_t slots);
int32_t  tgcomm_ring_push(void *ring, const tgcomm_msg_t *msg);
int32_t  tgcomm_ring_pop(void *ring, tgcomm_msg_t *out);
uint32_t tgcomm_ring_available(const void *ring);

void     tgcomm_cap_table_init(tgcomm_cap_table_t *table);
int32_t  tgcomm_cap_insert(tgcomm_cap_table_t *table, uint32_t cap,
                           uint64_t obj_id, uint8_t obj_type, uint32_t rights);
int32_t  tgcomm_cap_check(const tgcomm_cap_table_t *table, uint32_t cap,
                          uint32_t required_rights);
int32_t  tgcomm_cap_remove(tgcomm_cap_table_t *table, uint32_t cap);

/* ------------------------------------------------------------------ */
/* Convenience helpers (implemented in lib-C, call the Rust core)     */
/* ------------------------------------------------------------------ */
uint32_t tgcomm_ring_bytes(uint32_t slots);

static inline void tgcomm_build_request(tgcomm_msg_t *msg,
                                        tgcomm_layer_t layer,
                                        tgcomm_layer_t target,
                                        uint32_t opcode,
                                        uint32_t cap) {
    tgcomm_msg_init(msg);
    msg->kind   = TGCOMM_KIND_REQUEST;
    msg->layer  = (uint8_t)layer;
    msg->target = (uint8_t)target;
    msg->opcode = opcode;
    msg->cap    = cap;
}

#ifdef __cplusplus
}
#endif

#endif /* TGCOMM_H */

