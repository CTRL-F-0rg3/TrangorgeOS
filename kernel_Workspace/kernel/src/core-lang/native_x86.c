#include "native.h"

#if defined(__x86_64__)

typedef struct {
    uint8_t *buf;
    size_t len, cap;
} nb_t;

static void nb_u8(nb_t *d, uint8_t v)
{
    if (d->len < d->cap) d->buf[d->len++] = v;
}

static void nb_u32(nb_t *d, uint32_t v)
{
    for (int i = 0; i < 4; i++) nb_u8(d, (uint8_t)(v >> (i * 8)));
}

static void nb_u64(nb_t *d, uint64_t v)
{
    for (int i = 0; i < 8; i++) nb_u8(d, (uint8_t)(v >> (i * 8)));
}

static void mov_rax_u64(nb_t *d, uint64_t v)
{
    nb_u8(d, 0x48); nb_u8(d, 0xB8); nb_u64(d, v);
}

static void push_rax(nb_t *d) { nb_u8(d, 0x50); }
static void pop_rax(nb_t *d)  { nb_u8(d, 0x58); }
static void pop_rcx(nb_t *d)  { nb_u8(d, 0x59); }
static void push_rcx(nb_t *d) { nb_u8(d, 0x51); }

static void load_slot(nb_t *d, uint16_t a)   /* rax=[rbx+a*8]; push */
{
    nb_u8(d, 0x48); nb_u8(d, 0x8B); nb_u8(d, 0x83); nb_u32(d, (uint32_t)a * 8);
    push_rax(d);
}

static void store_slot(nb_t *d, uint16_t a)  /* pop rax; [rbx+a*8]=rax */
{
    pop_rax(d);
    nb_u8(d, 0x48); nb_u8(d, 0x89); nb_u8(d, 0x83); nb_u32(d, (uint32_t)a * 8);
}

static void load_glob(nb_t *d, uint16_t a)
{
    nb_u8(d, 0x48); nb_u8(d, 0x8B); nb_u8(d, 0x85); nb_u32(d, (uint32_t)a * 8);
    push_rax(d);
}

static void store_glob(nb_t *d, uint16_t a)
{
    pop_rax(d);
    nb_u8(d, 0x48); nb_u8(d, 0x89); nb_u8(d, 0x85); nb_u32(d, (uint32_t)a * 8);
}

typedef struct {
    uint32_t insn_idx;
    size_t patch_off;
} nb_patch_t;

static uint8_t g_nb_mem[16384];

void *cl_native_compile(cl_prog_t *P)
{
    if (P->main_fn < 0) return (void *)0;

    nb_t d = { g_nb_mem, 0, sizeof(g_nb_mem) };
    nb_patch_t patches[256];
    size_t npatches = 0;
    size_t off_by_insn[4096] = { 0 };

    /* prolog: rbx=slots, rbp=globals, r10=externs */
    nb_u8(&d, 0x48); nb_u8(&d, 0x89); nb_u8(&d, 0xFB);
    nb_u8(&d, 0x48); nb_u8(&d, 0x89); nb_u8(&d, 0xF5);
    nb_u8(&d, 0x49); nb_u8(&d, 0x89); nb_u8(&d, 0xD2);

    uint32_t pc = P->fns[P->main_fn].start;

    for (;;) {
        if (pc >= P->code_len) return (void *)0;

        off_by_insn[pc] = d.len;

        cl_insn_t in = P->code[pc++];

        switch (in.op) {
        case OP_CONST: {
            uint64_t v = 0;

            for (int i = 7; i >= 0; i--) {
                v = (v << 8) | P->consts[in.a].b[i];
            }

            mov_rax_u64(&d, v);
            push_rax(&d);
            break;
        }

        case OP_LOAD:  load_slot(&d, in.a); break;
        case OP_STORE: store_slot(&d, in.a); break;
        case OP_GLOAD: load_glob(&d, in.a); break;
        case OP_GSTORE: store_glob(&d, in.a); break;

        case OP_ADD:
            pop_rax(&d); pop_rcx(&d);
            nb_u8(&d, 0x48); nb_u8(&d, 0x01); nb_u8(&d, 0xC1);
            push_rcx(&d);
            break;

        case OP_SUB:
            pop_rax(&d); pop_rcx(&d);
            nb_u8(&d, 0x48); nb_u8(&d, 0x29); nb_u8(&d, 0xC1);
            push_rcx(&d);
            break;

        case OP_MUL:
            pop_rax(&d); pop_rcx(&d);
            nb_u8(&d, 0x48); nb_u8(&d, 0x0F); nb_u8(&d, 0xAF); nb_u8(&d, 0xC8);
            push_rcx(&d);
            break;

        case OP_EXTERN: {
            int n = in.b > 6 ? 6 : in.b;

            for (int i = n - 1; i >= 0; i--) {
                pop_rax(&d);

                switch (i) {
                case 0: nb_u8(&d,0x48); nb_u8(&d,0x89); nb_u8(&d,0xC7); break;
                case 1: nb_u8(&d,0x48); nb_u8(&d,0x89); nb_u8(&d,0xC6); break;
                case 2: nb_u8(&d,0x48); nb_u8(&d,0x89); nb_u8(&d,0xC2); break;
                case 3: nb_u8(&d,0x48); nb_u8(&d,0x89); nb_u8(&d,0xC1); break;
                case 4: nb_u8(&d,0x49); nb_u8(&d,0x89); nb_u8(&d,0xC0); break;
                case 5: nb_u8(&d,0x49); nb_u8(&d,0x89); nb_u8(&d,0xC1); break;
                }
            }

            /* rax = externs[a]; call rax; push rax */
            nb_u8(&d, 0x49); nb_u8(&d, 0x8B); nb_u8(&d, 0x82);
            nb_u32(&d, (uint32_t)in.a * 8);
            nb_u8(&d, 0xFF); nb_u8(&d, 0xD0);
            push_rax(&d);
            break;
        }

        case OP_POP: pop_rax(&d); break;

        case OP_JMP:
            nb_u8(&d, 0xE9);
            patches[npatches].insn_idx = in.c;
            patches[npatches].patch_off = d.len;
            npatches++;
            nb_u32(&d, 0);
            break;

        case OP_JZ:
            pop_rax(&d);
            nb_u8(&d, 0x48); nb_u8(&d, 0x85); nb_u8(&d, 0xC0);
            nb_u8(&d, 0x0F); nb_u8(&d, 0x84);
            patches[npatches].insn_idx = in.c;
            patches[npatches].patch_off = d.len;
            npatches++;
            nb_u32(&d, 0);
            break;

        case OP_RET:
        case OP_HALT:
            nb_u8(&d, 0x48); nb_u8(&d, 0x31); nb_u8(&d, 0xC0);
            nb_u8(&d, 0xC3);
            goto done;

        default:
            return (void *)0;   /* subset v1 */
        }
    }

done:
    for (size_t i = 0; i < npatches; i++) {
        uint32_t target = (uint32_t)off_by_insn[patches[i].insn_idx];
        uint32_t rel = (uint32_t)(target - (patches[i].patch_off + 4));

        uint8_t *p = d.buf + patches[i].patch_off;

        for (int k = 0; k < 4; k++) {
            p[k] = (uint8_t)(rel >> (k * 8));
        }
    }

    return d.buf;
}

size_t cl_native_size(void)
{
    return sizeof(g_nb_mem);
}

#else  /* aarch64 / riscv64 — na razie NULL, VM przejmuje */

void *cl_native_compile(cl_prog_t *P)
{
    (void)P;
    return (void *)0;
}

size_t cl_native_size(void)
{
    return 0;
}

#endif