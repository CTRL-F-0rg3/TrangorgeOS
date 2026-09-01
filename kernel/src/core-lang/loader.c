#include "loader.h"
#include "lexer.h"
#include "parser.h"
#include "sema.h"
#include "native.h"


extern cl_prog_t *cl_codegen(ast_node_t *prog, arena_t *ar);


cl_prog_t *cl_compile_source(const char *src, size_t len, arena_t *ar,
                             uint32_t *err_line, const char **err_msg)
{
    lexer_t l;
    size_t tcap = len / 3 + 128;
    token_t *toks = (token_t *)cl_arena_alloc(ar, tcap * sizeof(token_t));
    size_t n = 0;

    if (toks == (void *)0) {
        *err_line = 0;
        *err_msg = "arena full";
        return (void *)0;
    }

    cl_lexer_init(&l, src, len);

    if (cl_lex_all(&l, toks, tcap, &n) != 0) {
        *err_line = l.line;
        *err_msg = "lex error";
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

    if (s == (void *)0) {
        *err_line = 0;
        *err_msg = "arena full";
        return (void *)0;
    }

    cl_sema_run(s, prog);

    if (cl_sema_diag_count(s) > 0) {
        const cl_diag_t *d = cl_sema_diag(s, 0);

        *err_line = d->line;
        *err_msg = d->msg;
        return (void *)0;
    }

    cl_prog_t *P = cl_codegen(prog, ar);

    if (P == (void *)0) {
        *err_line = 0;
        *err_msg = "codegen failed";
        return (void *)0;
    }

    return P;
}


typedef struct {
    uint8_t *b;
    size_t cap;
    size_t o;
    bool ok;
} wr_t;

typedef struct {
    const uint8_t *b;
    size_t len;
    size_t o;
    bool ok;
} rd_t;

static void wr_u8(wr_t *w, uint8_t v)
{
    if (!w->ok || w->o >= w->cap) {
        w->ok = false;
        return;
    }

    w->b[w->o++] = v;
}

static void wr_u32(wr_t *w, uint32_t v)
{
    for (int i = 0; i < 4; i++) {
        wr_u8(w, (uint8_t)(v >> (i * 8)));
    }
}

static void wr_bytes(wr_t *w, const void *src, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        wr_u8(w, ((const uint8_t *)src)[i]);
    }
}

static uint8_t rd_u8(rd_t *r)
{
    if (!r->ok || r->o >= r->len) {
        r->ok = false;
        return 0;
    }

    return r->b[r->o++];
}

static uint32_t rd_u32(rd_t *r)
{
    uint32_t v = 0;

    for (int i = 0; i < 4; i++) {
        v |= (uint32_t)rd_u8(r) << (i * 8);
    }

    return v;
}

static void rd_bytes(rd_t *r, void *dst, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        ((uint8_t *)dst)[i] = rd_u8(r);
    }
}

size_t cl_save_bc(cl_prog_t *P, uint8_t *buf, size_t cap)
{
    wr_t w = { buf, cap, 0, true };

    wr_bytes(&w, "CLBC1", 5);

    wr_u32(&w, (uint32_t)P->code_len);

    for (size_t i = 0; i < P->code_len; i++) {
        const cl_insn_t *in = &P->code[i];

        wr_u8(&w, in->op);
        wr_u8(&w, in->bits);
        wr_u8(&w, (uint8_t)(in->a & 0xFF));
        wr_u8(&w, (uint8_t)(in->a >> 8));
        wr_u8(&w, (uint8_t)(in->b & 0xFF));
        wr_u8(&w, (uint8_t)(in->b >> 8));
        wr_u32(&w, in->c);
    }

    wr_u32(&w, (uint32_t)P->nconsts);

    for (size_t i = 0; i < P->nconsts; i++) {
        wr_bytes(&w, P->consts[i].b, 64);
        wr_u8(&w, P->const_bits[i]);
    }

    wr_u32(&w, (uint32_t)P->nfns);

    for (size_t i = 0; i < P->nfns; i++) {
        wr_u32(&w, P->fns[i].start);
        wr_u8(&w, (uint8_t)P->fns[i].nslots);
        wr_u8(&w, (uint8_t)P->fns[i].nargs);
        wr_bytes(&w, P->fn_names[i], 64);
    }

    wr_u32(&w, (uint32_t)P->nglobals);

    for (size_t i = 0; i < P->nglobals; i++) {
        wr_bytes(&w, P->gnames[i], 64);
        wr_u8(&w, P->gbits[i]);
        wr_u8(&w, P->gneg[i]);
        wr_bytes(&w, P->ginit[i].b, 64);
    }

    wr_u32(&w, (uint32_t)P->nexts);

    for (size_t i = 0; i < P->nexts; i++) {
        wr_bytes(&w, P->ext_names[i], 64);
    }

    wr_u32(&w, (uint32_t)P->main_fn);

    return w.ok ? w.o : 0;
}

cl_prog_t *cl_load_bc(const uint8_t *buf, size_t len, arena_t *ar)
{
    rd_t r = { buf, len, 0, true };

    uint8_t magic[5];

    rd_bytes(&r, magic, 5);

    if (!r.ok || magic[0] != 'C' || magic[1] != 'L' ||
        magic[2] != 'B' || magic[3] != 'C' || magic[4] != '1') {
        return (void *)0;
    }

    cl_prog_t *P = (cl_prog_t *)cl_arena_alloc(ar, sizeof(cl_prog_t));

    if (P == (void *)0) {
        return (void *)0;
    }

    P->ar = ar;
    P->code_cap = 4096;
    P->code = (cl_insn_t *)cl_arena_alloc(ar,
                P->code_cap * sizeof(cl_insn_t));

    if (P->code == (void *)0) {
        return (void *)0;
    }

    uint32_t cl = rd_u32(&r);

    if (cl > P->code_cap) {
        return (void *)0;
    }

    P->code_len = cl;

    for (size_t i = 0; i < P->code_len; i++) {
        cl_insn_t *in = &P->code[i];

        in->op = rd_u8(&r);
        in->bits = rd_u8(&r);
        in->a = (uint16_t)(rd_u8(&r) | ((uint16_t)rd_u8(&r) << 8));
        in->b = (uint16_t)(rd_u8(&r) | ((uint16_t)rd_u8(&r) << 8));
        in->c = rd_u32(&r);
    }

    P->nconsts = rd_u32(&r);

    if (P->nconsts > 64) {
        return (void *)0;
    }

    for (size_t i = 0; i < P->nconsts; i++) {
        rd_bytes(&r, P->consts[i].b, 64);
        P->const_bits[i] = rd_u8(&r);
    }

    P->nfns = rd_u32(&r);

    if (P->nfns > 16) {
        return (void *)0;
    }

    for (size_t i = 0; i < P->nfns; i++) {
        P->fns[i].start = rd_u32(&r);
        P->fns[i].nslots = rd_u8(&r);
        P->fns[i].nargs = rd_u8(&r);
        rd_bytes(&r, P->fn_names[i], 64);
    }

    P->nglobals = rd_u32(&r);

    if (P->nglobals > 64) {
        return (void *)0;
    }

    for (size_t i = 0; i < P->nglobals; i++) {
        rd_bytes(&r, P->gnames[i], 64);
        P->gbits[i] = rd_u8(&r);
        P->gneg[i] = rd_u8(&r);
        rd_bytes(&r, P->ginit[i].b, 64);
    }

    P->nexts = rd_u32(&r);

    if (P->nexts > 16) {
        return (void *)0;
    }

    for (size_t i = 0; i < P->nexts; i++) {
        rd_bytes(&r, P->ext_names[i], 64);
    }

    P->main_fn = (int)rd_u32(&r);

    if (!r.ok) {
        return (void *)0;
    }

    return P;
}



#ifdef CL_HOST
#include <stdio.h>

static long read_all(const char *path, uint8_t *buf, size_t cap)
{
    FILE *f = fopen(path, "rb");

    if (f == (void *)0) {
        return -1;
    }

    size_t n = fread(buf, 1, cap, f);

    fclose(f);

    return (long)n;
}
#else
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);

static long read_all(const char *path, uint8_t *buf, size_t cap)
{
    return (long)k_fs_read(path, buf, (uint32_t)cap);
}
#endif


cl_module_t *cl_load_path(const char *path, arena_t *ar, bool try_native)
{
    static uint8_t io[64 * 1024];

    long n = read_all(path, io, sizeof(io));

    if (n <= 0) {
        return (void *)0;
    }

    cl_module_t *m = (cl_module_t *)cl_arena_alloc(ar, sizeof(cl_module_t));

    if (m == (void *)0) {
        return (void *)0;
    }

    m->native = (void *)0;

    if (n > 5 && io[0] == 'C' && io[1] == 'L' && io[2] == 'B') {
        m->bc = cl_load_bc(io, (size_t)n, ar);
    } else {
        uint32_t el = 0;
        const char *em = (void *)0;

        m->bc = cl_compile_source((const char *)io, (size_t)n,
                                  ar, &el, &em);
    }

    if (m->bc == (void *)0) {
        return (void *)0;
    }

    if (try_native) {
        m->native = cl_native_compile(m->bc);
    }

    return m;
}