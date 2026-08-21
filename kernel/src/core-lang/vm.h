#ifndef CORELANG_VM_H
#define CORELANG_VM_H

#include "bc.h"

typedef enum {
    CL_OK = 0,
    CL_ERR_USE_AFTER_FREE,
    CL_ERR_DIV_ZERO,
    CL_ERR_STACK,
    CL_ERR_EXTERN,
    CL_ERR_BADOP,
    CL_ERR_DEPTH,
} cl_vm_err_t;

typedef uint64_t (*cl_ext_fn)(uint64_t, uint64_t, uint64_t,
                              uint64_t, uint64_t, uint64_t);

typedef struct {
    uint32_t pc;
    uint16_t fn_idx;
    cl_val_t slots[64];
    uint8_t slive[64];
} vm_frame_t;

typedef struct {
    cl_prog_t *prog;

    cl_val_t globals[64];
    uint8_t glive[64];

    cl_ext_fn externs[16];

    cl_val_t stack[256];
    int sp;

    vm_frame_t frames[8];
    int depth;

    cl_vm_err_t err;
    uint32_t err_pc;
} cl_vm_t;

void cl_vm_init(cl_vm_t *vm, cl_prog_t *prog);
int cl_vm_register_extern(cl_vm_t *vm, const char *name, cl_ext_fn fn);
cl_vm_err_t cl_vm_run(cl_vm_t *vm);

/* arytmetyka szeroka (używana też przez testy) */
int cl_wbytes(int bits);
void cl_mask(uint8_t *r, int w, int bits);
void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign);
void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w);
void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w);
void cl_divmod(const uint8_t *n, const uint8_t *d, int w,
               uint8_t *q, uint8_t *m);

#endif