#include "loader.h"
#include "vm.h"
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

extern void *cl_native_compile(cl_prog_t *P);

cl_prog_t *cl_compile_source(const char *src, size_t len, arena_t *ar,
                             uint32_t *err_line, const char **err_msg)
{
    lexer_t l;
    size_t tcap = len / 3 + 128;
    token_t *toks = (token_t *)cl_arena_alloc(ar, tcap * sizeof(token_t));
    size_t n = 0;

    cl_lexer_init(&l, src, len);

    if (cl_lex_all(&l, toks, tcap, &n) != 0) {
        *err_line = l.line;
        *err_msg = "lex";
        return (void *)0;
    }

    parser_t p;
    ast_node_t *prog = cl_parse(&p, toks, n, ar);

    if (prog == (void *)0) {
        *err_line = p.err_line;
        *err_msg = p.err;
        return (void *)0;
    }

    sema_ctx_t *s = cl_sema_new(ar);

    cl_sema_run(s, prog);

    if (cl_sema_diag_count(s) > 0) {
        const cl_diag_t *d = cl_sema_diag(s, 0);

        *err_line = d->line;
        *err_msg = d->msg;
        return (void *)0;
    }

    return cl_codegen(prog, ar);
}

/* image: "CLBC1" | u32 code_len | insns | u32 nconsts | consts+bits
   | u32 nfns | fns+names | u32 nglobals | names+bits+neg+init
   | u32 nexts | names | i32 main */

static void wr_u32(uint8_t *b, size_t *o, uint32_t v)
{
    for (int i = 0; i < 4; i++) b[(*o)++] = (uint8_t)(v >> (i * 8));
}

static uint32_t rd_u32(const uint8_t *b, size_t *o)
{
    uint32_t v = 0;

    for (int i = 0; i < 4; i++) v |= (uint32_t)b[(*o)++] << (i * 8);

    return v;
}

size_t cl_save_bc(cl_prog_t *P, uint8_t *buf, size_t cap)
{
    size_t o = 0;

    const uint8_t magic[5] = { 'C', 'L', 'B', 'C', '1' };

    for (int i = 0; i < 5; i++) buf[o++] = magic[i];

    wr_u32(buf, &o, (uint32_t)P->code_len);

    for (size_t i = 0; i < P->code_len; i++) {
        const cl_insn_t *in = &P->code[i];

        buf[o++] = in->op;
        buf[o++] = in->bits;
        buf[o++] = (uint8_t)(in->a & 0xFF);
        buf[o++] = (uint8_t)(in->a >> 8);
        buf[o++] = (uint8_t)(in->b & 0xFF);
        buf[o++] = (uint8_t)(in->b >> 8);
        wr_u32(buf, &o, in->c);
    }

    wr_u32(buf, &o, (uint32_t)P->nconsts);

    for (size_t i = 0; i < P->nconsts; i++) {
        for (int k = 0; k < 64; k++) buf[o++] = P->consts[i].b[k];
        buf[o++] = P->const_bits[i];
    }

    wr_u32(buf, &o, (uint32_t)P->nfns);

    for (size_t i = 0; i < P->nfns; i++) {
        wr_u32(buf, &o, P->fns[i].start);
        buf[o++] = (uint8_t)(P->fns[i].nslots & 0xFF);
        buf[o++] = (uint8_t)(P->fns[i].nargs & 0xFF);

        for (int k = 0; k < 64; k++) buf[o++] = (uint8_t)P->fn_names[i][k];
    }

    wr_u32(buf, &o, (uint32_t)P->nglobals);

    for (size_t i = 0; i < P->nglobals; i++) {
        for (int k = 0; k < 64; k++) buf[o++] = (uint8_t)P->gnames[i][k];

        buf[o++] = P->gbits[i];
        buf[o++] = P->gneg[i];

        for (int k = 0; k < 64; k++) buf[o++] = P->ginit[i].b[k];
    }

    wr_u32(buf, &o, (uint32_t)P->nexts);

    for (size_t i = 0; i < P->nexts; i++) {
        for (int k = 0; k < 64; k++) buf[o++] = (uint8_t)P->ext_names[i][k];
    }

    wr_u32(buf, &o, (uint32_t)P->main_fn);

    return o <= cap ? o : 0;
}

cl_prog_t *cl_load_bc(const uint8_t *buf, size_t len, arena_t *ar)
{
    size_t o = 0;

    if (len < 5 || buf[0] != 'C' || buf[1] != 'L' ||
        buf[2] != 'B' || buf[3] != 'C' || buf[4] != '1') {
        return (void *)0;
    }

    o = 5;

    cl_prog_t *P = (cl_prog_t *)cl_arena_alloc(ar, sizeof(cl_prog_t));

    if (P == (void *)0) return (void *)0;

    P->ar = ar;
    P->code_cap = 4096;
    P->code = (cl_insn_t *)cl_arena_alloc(ar, P->code_cap * sizeof(cl_insn_t));

    P->code_len = rd_u32(buf, &o);

    for (size_t i = 0; i < P->code_len; i++) {
        cl_insn_t *in = &P->code[i];

        in->op = buf[o++];
        in->bits = buf[o++];
        in->a = (uint16_t)(buf[o] | (buf[o + 1] << 8)); o += 2;
        in->b = (uint16_t)(buf[o] | (buf[o + 1] << 8)); o += 2;
        in->c = rd_u32(buf, &o);
    }

    P->nconsts = rd_u32(buf, &o);

    for (size_t i = 0; i < P->nconsts; i++) {
        for (int k = 0; k < 64; k++) P->consts[i].b[k] = buf[o++];
        P->const_bits[i] = buf[o++];
    }

    P->nfns = rd_u32(buf, &o);

    for (size_t i = 0; i < P->nfns; i++) {
        P->fns[i].start = rd_u32(buf, &o);
        P->fns[i].nslots = buf[o++];
        P->fns[i].nargs = buf[o++];

        for (int k = 0; k < 64; k++) P->fn_names[i][k] = (char)buf[o++];
    }

    P->nglobals = rd_u32(buf, &o);

    for (size_t i = 0; i < P->nglobals; i++) {
        for (int k = 0; k < 64; k++) P->gnames[i][k] = (char)buf[o++];

        P->gbits[i] = buf[o++];
        P->gneg[i] = buf[o++];

        for (int k = 0; k < 64; k++) P->ginit[i].b[k] = buf[o++];
    }

    P->nexts = rd_u32(buf, &o);

    for (size_t i = 0; i < P->nexts; i++) {
        for (int k = 0; k < 64; k++) P->ext_names[i][k] = (char)buf[o++];
    }

    P->main_fn = (int)rd_u32(buf, &o);

    return P;
}

#ifdef CL_KERNEL
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);

static long read_all(const char *path, uint8_t *buf, size_t cap)
{
    return (long)k_fs_read(path, buf, (uint32_t)cap);
}
#else
#include <stdio.h>

static long read_all(const char *path, uint8_t *buf, size_t cap)
{
    FILE *f = fopen(path, "rb");

    if (f == (void *)0) return -1;

    size_t n = fread(buf, 1, cap, f);

    fclose(f);

    return (long)n;
}
#endif

cl_module_t *cl_load_path(const char *path, arena_t *ar, bool try_native)
{
    static uint8_t io[64 * 1024];

    long n = read_all(path, io, sizeof(io));

    if (n <= 0) return (void *)0;

    cl_module_t *m = (cl_module_t *)cl_arena_alloc(ar, sizeof(cl_module_t));

    if (m == (void *)0) return (void *)0;

    m->native = (void *)0;

    if (n > 5 && io[0] == 'C' && io[1] == 'L' && io[2] == 'B') {
        m->bc = cl_load_bc(io, (size_t)n, ar);
    } else {
        uint32_t el = 0;
        const char *em = (void *)0;

        m->bc = cl_compile_source((const char *)io, (size_t)n, ar, &el, &em);
    }

    if (m->bc == (void *)0) return (void *)0;

    if (try_native) {
        m->native = cl_native_compile(m->bc);
    }

    return m;
}