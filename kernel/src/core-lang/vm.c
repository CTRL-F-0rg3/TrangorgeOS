#include "vm.h"

int cl_wbytes(int bits)
{
    if (bits <= 8) return 1;
    if (bits <= 16) return 2;
    if (bits <= 32) return 4;
    if (bits <= 64) return 8;
    if (bits <= 128) return 16;
    if (bits <= 256) return 32;
    return 64;
}

void cl_mask(uint8_t *r, int w, int bits)
{
    int full = bits / 8;
    int rem = bits % 8;

    for (int i = full + (rem ? 1 : 0); i < w; i++) {
        r[i] = 0;
    }

    if (rem) {
        r[full] &= (uint8_t)((1 << rem) - 1);
    }
}

void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
{
    int c = 0;

    for (int i = 0; i < w; i++) {
        int s = a[i] + b[i] + c;

        r[i] = (uint8_t)s;
        c = s >> 8;
    }
}

void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
{
    int br = 0;

    for (int i = 0; i < w; i++) {
        int s = a[i] - b[i] - br;

        r[i] = (uint8_t)s;
        br = (s < 0) ? 1 : 0;
    }
}

void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
{
    uint8_t t[64] = { 0 };

    for (int i = 0; i < w; i++) {
        int c = 0;

        for (int j = 0; j + i < w; j++) {
            int s = t[i + j] + a[i] * b[j] + c;

            t[i + j] = (uint8_t)s;
            c = s >> 8;
        }
    }

    for (int i = 0; i < w; i++) {
        r[i] = t[i];
    }
}

static int bit_get(const uint8_t *v, int i)
{
    return (v[i / 8] >> (i % 8)) & 1;
}

static void bit_set(uint8_t *v, int i)
{
    v[i / 8] |= (uint8_t)(1 << (i % 8));
}

static int cmp_u(const uint8_t *a, const uint8_t *b, int w)
{
    for (int i = w - 1; i >= 0; i--) {
        if (a[i] != b[i]) {
            return a[i] > b[i] ? 1 : -1;
        }
    }

    return 0;
}

int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign)
{
    if (!sign) {
        return cmp_u(a, b, w);
    }

    uint8_t x[64], y[64];

    for (int i = 0; i < w; i++) {
        x[i] = a[i];
        y[i] = b[i];
    }

    int sb = w * 8 - 1;

    x[w - 1] ^= 0x80;
    y[w - 1] ^= 0x80;

    (void)sb;

    return cmp_u(x, y, w);
}

void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w)
{
    uint8_t t[64] = { 0 };

    for (int i = 0; i < w * 8; i++) {
        if (bit_get(a, i) && (int)(i + n) < w * 8) {
            bit_set(t, i + (int)n);
        }
    }

    for (int i = 0; i < w; i++) {
        r[i] = t[i];
    }
}

void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w)
{
    uint8_t t[64] = { 0 };

    for (int i = 0; i < w * 8; i++) {
        if (bit_get(a, i) && i >= (int)n) {
            bit_set(t, i - (int)n);
        }
    }

    for (int i = 0; i < w; i++) {
        r[i] = t[i];
    }
}

void cl_divmod(const uint8_t *n, const uint8_t *d, int w,
               uint8_t *q, uint8_t *m)
{
    uint8_t rem[64] = { 0 };
    uint8_t quo[64] = { 0 };

    for (int i = w * 8 - 1; i >= 0; i--) {
        uint8_t shl[64];
        cl_shl(shl, rem, 1, w);

        for (int k = 0; k < w; k++) {
            rem[k] = shl[k];
        }

        if (bit_get(n, i)) {
            rem[0] |= 1;
        }

        if (cmp_u(rem, d, w) >= 0) {
            uint8_t sub[64];
            cl_sub(sub, rem, d, w);

            for (int k = 0; k < w; k++) {
                rem[k] = sub[k];
            }

            bit_set(quo, i);
        }
    }

    for (int i = 0; i < w; i++) {
        q[i] = quo[i];
        m[i] = rem[i];
    }
}

/* ---- VM ---- */

void cl_vm_init(cl_vm_t *vm, cl_prog_t *prog)
{
    vm->prog = prog;
    vm->sp = 0;
    vm->depth = 0;
    vm->err = CL_OK;

    for (size_t i = 0; i < 64; i++) {
        vm->globals[i] = prog->ginit[i];
        vm->glive[i] = 1;
    }

    for (size_t i = 0; i < 16; i++) {
        vm->externs[i] = (cl_ext_fn)0;
    }
}

int cl_vm_register_extern(cl_vm_t *vm, const char *name, cl_ext_fn fn)
{
    for (size_t i = 0; i < vm->prog->nexts; i++) {
        bool same = true;

        for (int k = 0; vm->prog->ext_names[i][k] || name[k]; k++) {
            if (vm->prog->ext_names[i][k] != name[k]) {
                same = false;
                break;
            }
        }

        if (same) {
            vm->externs[i] = fn;
            return 0;
        }
    }

    return -1;
}

static void push(cl_vm_t *vm, const cl_val_t *v)
{
    if (vm->sp < 256) {
        vm->stack[vm->sp++] = *v;
    } else {
        vm->err = CL_ERR_STACK;
    }
}

static cl_val_t pop(cl_vm_t *vm)
{
    if (vm->sp > 0) {
        return vm->stack[--vm->sp];
    }

    vm->err = CL_ERR_STACK;

    cl_val_t z = { { 0 } };
    return z;
}

static uint64_t val_u64(const cl_val_t *v)
{
    uint64_t x = 0;

    for (int i = 7; i >= 0; i--) {
        x = (x << 8) | v->b[i];
    }

    return x;
}

cl_vm_err_t cl_vm_run(cl_vm_t *vm)
{
    cl_prog_t *P = vm->prog;

    vm->depth = 0;

    if (P->main_fn < 0) {
        return CL_ERR_BADOP;
    }

    vm->frames[0].pc = P->fns[P->main_fn].start;
    vm->frames[0].fn_idx = (uint16_t)P->main_fn;

    for (int i = 0; i < 64; i++) {
        vm->frames[0].slive[i] = 0;
    }

    vm->depth = 1;

    while (vm->depth > 0 && vm->err == CL_OK) {
        vm_frame_t *f = &vm->frames[vm->depth - 1];

        if (f->pc >= P->code_len) {
            break;
        }

        cl_insn_t in = P->code[f->pc++];

        int w = cl_wbytes(in.bits);
        bool sign = false;   /* v1: signedness z gneg/slotów pomijana w op */

        cl_val_t a, b, r;

        switch (in.op) {
        case OP_NOP: break;

        case OP_CONST:
            r = P->consts[in.a];
            cl_mask(r.b, w, in.bits);
            push(vm, &r);
            break;

        case OP_LOAD:
            if (!f->slive[in.a]) {
                vm->err = CL_ERR_USE_AFTER_FREE;
                break;
            }
            push(vm, &f->slots[in.a]);
            break;

        case OP_STORE:
            a = pop(vm);
            cl_mask(a.b, w, in.bits);
            f->slots[in.a] = a;
            f->slive[in.a] = 1;
            break;

        case OP_GLOAD:
            if (!vm->glive[in.a]) {
                vm->err = CL_ERR_USE_AFTER_FREE;
                break;
            }
            push(vm, &vm->globals[in.a]);
            break;

        case OP_GSTORE:
            a = pop(vm);
            cl_mask(a.b, w, in.bits);
            vm->globals[in.a] = a;
            vm->glive[in.a] = 1;
            break;

        case OP_ADD: case OP_SUB: case OP_MUL:
        case OP_DIV: case OP_MOD:
        case OP_AND: case OP_OR: case OP_XOR:
        case OP_SHL: case OP_SHR:
            b = pop(vm);
            a = pop(vm);

            for (int i = 0; i < 64; i++) r.b[i] = 0;

            switch (in.op) {
            case OP_ADD: cl_add(r.b, a.b, b.b, w); break;
            case OP_SUB: cl_sub(r.b, a.b, b.b, w); break;
            case OP_MUL: cl_mul(r.b, a.b, b.b, w); break;
            case OP_AND: for (int i = 0; i < w; i++) r.b[i] = a.b[i] & b.b[i]; break;
            case OP_OR:  for (int i = 0; i < w; i++) r.b[i] = a.b[i] | b.b[i]; break;
            case OP_XOR: for (int i = 0; i < w; i++) r.b[i] = a.b[i] ^ b.b[i]; break;
            case OP_SHL: cl_shl(r.b, a.b, (uint32_t)b.b[0], w); break;
            case OP_SHR: cl_shr(r.b, a.b, (uint32_t)b.b[0], w); break;
            case OP_DIV: case OP_MOD: {
                uint8_t q[64], m[64];
                bool zero = true;

                for (int i = 0; i < w; i++) {
                    if (b.b[i]) zero = false;
                }

                if (zero) {
                    vm->err = CL_ERR_DIV_ZERO;
                    break;
                }

                cl_divmod(a.b, b.b, w, q, m);

                for (int i = 0; i < w; i++) {
                    r.b[i] = (in.op == OP_DIV) ? q[i] : m[i];
                }
                break;
            }
            }

            cl_mask(r.b, w, in.bits);
            push(vm, &r);
            break;

        case OP_NEG:
            a = pop(vm);
            {
                uint8_t z[64] = { 0 };
                cl_sub(r.b, z, a.b, w);
                cl_mask(r.b, w, in.bits);
            }
            push(vm, &r);
            break;

        case OP_NOT:
            a = pop(vm);
            for (int i = 0; i < w; i++) r.b[i] = ~a.b[i];
            cl_mask(r.b, w, in.bits);
            push(vm, &r);
            break;

        case OP_EQ: case OP_NE: case OP_LT: case OP_GT:
        case OP_LE: case OP_GE:
            b = pop(vm);
            a = pop(vm);
            {
                int c = cl_cmp(a.b, b.b, w, sign);
                bool t = false;

                switch (in.op) {
                case OP_EQ: t = (c == 0); break;
                case OP_NE: t = (c != 0); break;
                case OP_LT: t = (c < 0); break;
                case OP_GT: t = (c > 0); break;
                case OP_LE: t = (c <= 0); break;
                case OP_GE: t = (c >= 0); break;
                }

                for (int i = 0; i < 64; i++) r.b[i] = 0;
                r.b[0] = t ? 1 : 0;
            }
            push(vm, &r);
            break;

        case OP_JMP:
            f->pc = in.c;
            break;

        case OP_JZ:
            a = pop(vm);
            if (a.b[0] == 0) f->pc = in.c;
            break;

        case OP_JNZ:
            a = pop(vm);
            if (a.b[0] != 0) f->pc = in.c;
            break;

        case OP_POP:
            pop(vm);
            break;

        case OP_FREE:
            if (in.a == 1) {
                vm->glive[in.b] = 0;
                for (int i = 0; i < 64; i++) vm->globals[in.b].b[i] = 0;
            } else {
                f->slive[in.b] = 0;
                for (int i = 0; i < 64; i++) f->slots[in.b].b[i] = 0;
            }
            break;

        case OP_SHARED:
            if (!vm->glive[in.a]) {
                vm->err = CL_ERR_USE_AFTER_FREE;
            }
            break;

        case OP_EXTERN: {
            if (in.a >= 16 || vm->externs[in.a] == (cl_ext_fn)0) {
                vm->err = CL_ERR_EXTERN;
                break;
            }

            uint64_t args[6] = { 0 };
            int n = in.b > 6 ? 6 : in.b;

            for (int i = n - 1; i >= 0; i--) {
                cl_val_t v = pop(vm);
                args[i] = val_u64(&v);
            }

            uint64_t ret = vm->externs[in.a](args[0], args[1], args[2],
                                             args[3], args[4], args[5]);

            for (int i = 0; i < 64; i++) r.b[i] = 0;
            for (int i = 0; i < 8; i++) r.b[i] = (uint8_t)(ret >> (i * 8));

            push(vm, &r);
            break;
        }

        case OP_CALL: {
            if (vm->depth >= 8) {
                vm->err = CL_ERR_DEPTH;
                break;
            }

            cl_val_t argv[8];
            int n = in.b > 8 ? 8 : in.b;

            for (int i = n - 1; i >= 0; i--) {
                argv[i] = pop(vm);
            }

            vm_frame_t *nf = &vm->frames[vm->depth++];

            nf->pc = P->fns[in.a].start;
            nf->fn_idx = in.a;

            for (int i = 0; i < 64; i++) {
                nf->slive[i] = 0;
            }

            for (int i = 0; i < n; i++) {
                nf->slots[i] = argv[i];
                nf->slive[i] = 1;
            }
            break;
        }

        case OP_RET:
            vm->depth--;
            break;

        case OP_HALT:
            vm->depth = 0;
            break;

        default:
            vm->err = CL_ERR_BADOP;
            break;
        }

        vm->err_pc = f->pc;
    }

    return vm->err;
}