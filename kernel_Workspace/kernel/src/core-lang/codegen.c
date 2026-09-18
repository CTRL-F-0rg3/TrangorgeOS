#include "bc.h"
#include "vm.h"

typedef struct {
    char name[64];
    uint16_t slot;
    uint8_t bits;
} cg_local_t;

typedef struct {
    cl_prog_t *P;
    cg_local_t locals[32];
    uint16_t nlocals;
    uint16_t maxslots;
} cg_t;

static uint32_t emit(cg_t *g, uint8_t op, uint8_t bits,
                     uint16_t a, uint16_t b, uint32_t c)
{
    cl_prog_t *P = g->P;

    if (P->code_len >= P->code_cap) {
        return 0;
    }

    cl_insn_t in;

    in.op = op;
    in.bits = bits;
    in.a = a;
    in.b = b;
    in.c = c;

    P->code[P->code_len] = in;

    return (uint32_t)P->code_len++;
}

static int find_global(cl_prog_t *P, const char *name)
{
    for (size_t i = 0; i < P->nglobals; i++) {
        bool same = true;

        for (int k = 0; P->gnames[i][k] || name[k]; k++) {
            if (P->gnames[i][k] != name[k]) {
                same = false;
                break;
            }
        }

        if (same) return (int)i;
    }

    return -1;
}

static int find_local(cg_t *g, const char *name)
{
    for (int i = 0; i < g->nlocals; i++) {
        bool same = true;

        for (int k = 0; g->locals[i].name[k] || name[k]; k++) {
            if (g->locals[i].name[k] != name[k]) {
                same = false;
                break;
            }
        }

        if (same) return i;
    }

    return -1;
}

static uint16_t add_local(cg_t *g, const char *name, uint8_t bits)
{
    int i = find_local(g, name);

    if (i >= 0) {
        return g->locals[i].slot;
    }

    if (g->nlocals >= 32) {
        return 0;
    }

    for (int k = 0; name[k] && k < 63; k++) {
        g->locals[g->nlocals].name[k] = name[k];
        g->locals[g->nlocals].name[k + 1] = '\0';
    }

    g->locals[g->nlocals].bits = bits;
    g->locals[g->nlocals].slot = g->maxslots;

    return g->locals[g->nlocals++].slot;
}

static uint32_t add_const(cg_t *g, uint64_t v, uint8_t bits, bool neg)
{
    cl_prog_t *P = g->P;

    if (P->nconsts >= 64) return 0;

    for (int i = 0; i < 64; i++) P->consts[P->nconsts].b[i] = 0;

    int w = cl_wbytes(bits);

    for (int i = 0; i < 8; i++) {
        P->consts[P->nconsts].b[i] = (uint8_t)(v >> (i * 8));
    }

    if (neg) {
        uint8_t z[64] = { 0 };
        uint8_t sub[64];
        cl_sub(sub, z, P->consts[P->nconsts].b, w);
        for (int i = 0; i < w; i++) P->consts[P->nconsts].b[i] = sub[i];
    }

    cl_mask(P->consts[P->nconsts].b, w, bits);

    P->const_bits[P->nconsts] = bits;

    return (uint32_t)P->nconsts++;
}

static void gen_expr(cg_t *g, ast_node_t *n, uint8_t bits);

static void gen_expr(cg_t *g, ast_node_t *n, uint8_t bits)
{
    cl_prog_t *P = g->P;

    if (n == (void *)0) return;

    switch (n->kind) {
    case ND_NUM:
        emit(g, OP_CONST, bits, (uint16_t)add_const(g, n->value, bits, false), 0, 0);
        break;

    case ND_VAR: {
        int li = find_local(g, n->name);

        if (li >= 0) {
            emit(g, OP_LOAD, bits, g->locals[li].slot, 0, 0);
        } else {
            int gi = find_global(P, n->name);

            if (gi >= 0) {
                emit(g, OP_GLOAD, bits, (uint16_t)gi, 0, 0);
            } else {
                emit(g, OP_CONST, bits, (uint16_t)add_const(g, 0, bits, false), 0, 0);
            }
        }
        break;
    }

    case ND_POOL:
        if (n->pool_kind == 0) {
            emit(g, OP_GLOAD, bits, (uint16_t)find_global(P, n->name), 0, 0);
        } else if (n->pool_kind == 1) {
            int gi = (uint16_t)find_global(P, n->name);
            emit(g, OP_GLOAD, bits, (uint16_t)gi, 0, 0);
            emit(g, OP_NEG, bits, 0, 0, 0);
            emit(g, OP_GSTORE, bits, (uint16_t)gi, 0, 0);
            emit(g, OP_GLOAD, bits, (uint16_t)gi, 0, 0);
        } else {
            int gi = (uint16_t)find_global(P, n->name);
            emit(g, OP_SHARED, bits, (uint16_t)gi, 0, 0);
            emit(g, OP_GLOAD, bits, (uint16_t)gi, 0, 0);
        }
        break;

    case ND_CALLIDX: {
        int li = find_local(g, n->name);

        if (li >= 0) {
            emit(g, OP_LOAD, bits, g->locals[li].slot, 0, 0);
        } else {
            emit(g, OP_GLOAD, bits, (uint16_t)find_global(P, n->name), 0, 0);
        }
        break;
    }

    case ND_UN:
        gen_expr(g, n->a, bits);

        if (n->op == '-') {
            emit(g, OP_NEG, bits, 0, 0, 0);
        } else {
            emit(g, OP_NOT, bits, 0, 0, 0);
        }
        break;

    case ND_BIN: {
        uint8_t op = 0;

        switch (n->op) {
        case '+': op = OP_ADD; break;
        case '-': op = OP_SUB; break;
        case '*': op = OP_MUL; break;
        case '/': op = OP_DIV; break;
        case '%': op = OP_MOD; break;
        case '&': op = OP_AND; break;
        case '|': op = OP_OR; break;
        case '^': op = OP_XOR; break;
        case TK_SHL: op = OP_SHL; break;
        case TK_EQ: op = OP_EQ; break;
        case TK_NEQ: op = OP_NE; break;
        case TK_LT: op = OP_LT; break;
        case TK_GT: op = OP_GT; break;
        case TK_LEQ: op = OP_LE; break;
        case TK_GEQ: op = OP_GE; break;
        default: op = OP_ADD; break;
        }

        gen_expr(g, n->a, bits);
        gen_expr(g, n->b, bits);
        emit(g, op, bits, 0, 0, 0);
        break;
    }

    case ND_ASSIGN: {
        gen_expr(g, n->b, bits);

        if (n->a->kind == ND_VAR) {
            int li = find_local(g, n->a->name);

            if (li >= 0) {
                emit(g, OP_STORE, bits, g->locals[li].slot, 0, 0);
            } else {
                emit(g, OP_GSTORE, bits, (uint16_t)find_global(P, n->a->name), 0, 0);
            }
        }
        break;
    }

    case ND_MEMBER:
        gen_expr(g, n->a, bits);
        break;

    case ND_CALL: {
        if (n->a != (void *)0 && n->a->kind == ND_VAR) {
            int ei = -1;

            for (size_t i = 0; i < P->nexts; i++) {
                bool same = true;

                for (int k = 0; P->ext_names[i][k] || n->a->name[k]; k++) {
                    if (P->ext_names[i][k] != n->a->name[k]) {
                        same = false;
                        break;
                    }
                }

                if (same) { ei = (int)i; break; }
            }

            for (size_t i = 0; i < n->nlist; i++) {
                gen_expr(g, n->list[i], 64);
            }

            if (ei >= 0) {
                emit(g, OP_EXTERN, 64, (uint16_t)ei, (uint16_t)n->nlist, 0);
            } else {
                int fi = -1;

                for (size_t i = 0; i < P->nfns; i++) {
                    bool same = true;

                    for (int k = 0; P->fn_names[i][k] || n->a->name[k]; k++) {
                        if (P->fn_names[i][k] != n->a->name[k]) {
                            same = false;
                            break;
                        }
                    }

                    if (same) { fi = (int)i; break; }
                }

                if (fi >= 0) {
                    emit(g, OP_CALL, bits, (uint16_t)fi, (uint16_t)n->nlist, 0);
                }
            }
        }
        break;
    }

    default:
        break;
    }
}

static void gen_stmt(cg_t *g, ast_node_t *n)
{
    cl_prog_t *P = g->P;

    switch (n->kind) {
    case ND_LET: {
        uint16_t slot = add_local(g, n->name, n->type.bits);

        gen_expr(g, n->a, n->type.bits);
        emit(g, OP_STORE, n->type.bits, slot, 0, 0);
        break;
    }

    case ND_SET_FREE: {
        int li = find_local(g, n->name);

        if (li >= 0) {
            emit(g, OP_FREE, 0, 0, g->locals[li].slot, 0);
        } else {
            emit(g, OP_FREE, 0, 1, (uint16_t)find_global(P, n->name), 0);
        }
        break;
    }

    case ND_IF: {
        if (n->a != (void *)0) {
            gen_expr(g, n->a, 1);

            uint32_t jz = emit(g, OP_JZ, 0, 0, 0, 0);

            gen_stmt(g, n->b);

            if (n->c != (void *)0) {
                uint32_t jmp = emit(g, OP_JMP, 0, 0, 0, 0);

                P->code[jz].c = (uint32_t)P->code_len;
                gen_stmt(g, n->c);
                P->code[jmp].c = (uint32_t)P->code_len;
            } else {
                P->code[jz].c = (uint32_t)P->code_len;
            }
        } else {
            gen_stmt(g, n->b);
        }
        break;
    }

    case ND_WHEN: {
        uint16_t tslot = add_local(g, n->name, 64);

        int li2 = find_local(g, n->name);

        if (li2 >= 0 && g->locals[li2].slot != tslot) {
            emit(g, OP_LOAD, 64, g->locals[li2].slot, 0, 0);
            emit(g, OP_STORE, 64, tslot, 0, 0);
        } else if (li2 < 0) {
            emit(g, OP_GLOAD, 64, (uint16_t)find_global(P, n->name), 0, 0);
            emit(g, OP_STORE, 64, tslot, 0, 0);
        }

        for (size_t i = 0; i < n->nlist; i++) {
        }
        break;
    }

    case ND_BLOCK:
        for (size_t i = 0; i < n->nlist; i++) {
            gen_stmt(g, n->list[i]);
        }
        break;

    default:
        gen_expr(g, n, 64);
        emit(g, OP_POP, 0, 0, 0, 0);
        break;
    }
}

cl_prog_t *cl_codegen(ast_node_t *prog, arena_t *ar)
{
    cl_prog_t *P = (cl_prog_t *)cl_arena_alloc(ar, sizeof(cl_prog_t));

    if (P == (void *)0) return (void *)0;

    P->ar = ar;
    P->main_fn = -1;

    P->code_cap = 4096;
    P->code = (cl_insn_t *)cl_arena_alloc(ar, P->code_cap * sizeof(cl_insn_t));

    if (P->code == (void *)0) return (void *)0;

    for (size_t i = 0; i < prog->nlist; i++) {
        ast_node_t *n = prog->list[i];

        if (n->kind == ND_LET && P->nglobals < 64) {
            for (int k = 0; n->name[k] && k < 63; k++) {
                P->gnames[P->nglobals][k] = n->name[k];
                P->gnames[P->nglobals][k + 1] = '\0';
            }

            P->gbits[P->nglobals] = n->type.bits;
            P->gneg[P->nglobals] = n->type.neg;

            for (int k = 0; k < 8; k++) {
                P->ginit[P->nglobals].b[k] =
                    (n->a != (void *)0 && n->a->kind == ND_NUM)
                    ? (uint8_t)(n->a->value >> (k * 8)) : 0;
            }

            cl_mask(P->ginit[P->nglobals].b,
                    cl_wbytes(n->type.bits), n->type.bits);

            P->nglobals++;
        } else if (n->kind == ND_EXTERN) {
            for (size_t k = 0; k < n->nlist && P->nexts < 16; k++) {
                for (int x = 0; n->list[k]->name[x] && x < 63; x++) {
                    P->ext_names[P->nexts][x] = n->list[k]->name[x];
                    P->ext_names[P->nexts][x + 1] = '\0';
                }
                P->nexts++;
            }
        } else if (n->kind == ND_FN && P->nfns < 16) {
            for (int k = 0; n->name[k] && k < 63; k++) {
                P->fn_names[P->nfns][k] = n->name[k];
                P->fn_names[P->nfns][k + 1] = '\0';
            }

            P->fns[P->nfns].nargs = (uint16_t)n->nlist;
            P->nfns++;
        }
    }

    for (size_t i = 0; i < prog->nlist; i++) {
        ast_node_t *n = prog->list[i];

        if (n->kind != ND_FN) continue;

        int fi = -1;

        for (size_t k = 0; k < P->nfns; k++) {
            bool same = true;

            for (int x = 0; P->fn_names[k][x] || n->name[x]; x++) {
                if (P->fn_names[k][x] != n->name[x]) {
                    same = false;
                    break;
                }
            }

            if (same) { fi = (int)k; break; }
        }

        cg_t g = { 0 };

        g.P = P;

        for (size_t k = 0; k < n->nlist; k++) {
            add_local(&g, n->list[k]->name, n->list[k]->type.bits);
        }

        P->fns[fi].start = (uint32_t)P->code_len;

        gen_stmt(&g, n->a);

        emit(&g, OP_RET, 0, 0, 0, 0);

        P->fns[fi].nslots = g.maxslots;

        bool is_main = true;
        const char *m = "main";

        for (int k = 0; m[k] || n->name[k]; k++) {
            if (m[k] != n->name[k]) { is_main = false; break; }
        }

        if (is_main) {
            P->main_fn = fi;
            P->code[P->code_len - 1].op = OP_HALT;
        }
    }

    return P;
}