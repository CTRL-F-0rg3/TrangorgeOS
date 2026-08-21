#include "sema.h"

enum {
    SYM_VAR, SYM_STATIC, SYM_FN, SYM_EXTERN,
    SYM_ENUM, SYM_DATA,
};

typedef struct symbol {
    char name[64];
    uint8_t kind;
    cl_type_t type;
    bool live;
    bool pool_ref;
    bool freed;
    uint64_t value;
    int depth;
    uint8_t nparams;
    cl_type_t params[8];
    ast_node_t *node;
    struct symbol *next;
} symbol_t;

typedef struct scope {
    symbol_t *head;
} scope_t;

struct sema_ctx {
    arena_t *ar;
    scope_t scopes[16];
    int top;

    cl_diag_t diags[64];
    size_t ndiags;

    /* $@ pairing per statement */
    char sh_name[2][64];
    cl_type_t sh_type[2];
    uint32_t sh_line[2];
    int sh_count;
};

sema_ctx_t *cl_sema_new(arena_t *ar)
{
    sema_ctx_t *s = (sema_ctx_t *)cl_arena_alloc(ar, sizeof(sema_ctx_t));

    if (s == (void *)0) {
        return (void *)0;
    }

    s->ar = ar;
    s->top = 0;

    return s;
}

static void diag(sema_ctx_t *s, uint32_t line, const char *msg,
                 const char *name, bool warn)
{
    if (s->ndiags < 64) {
        cl_diag_t *d = &s->diags[s->ndiags++];

        d->line = line;
        d->msg = msg;
        d->is_warn = warn;
        d->name[0] = '\0';

        if (name != (void *)0) {
            for (int i = 0; name[i] && i < 63; i++) {
                d->name[i] = name[i];
                d->name[i + 1] = '\0';
            }
        }
    }
}

static void push_scope(sema_ctx_t *s)
{
    if (s->top < 15) {
        s->top++;
        s->scopes[s->top].head = (void *)0;
    }
}

static void pop_scope(sema_ctx_t *s)
{
    /* warn: lokalna zmienna żywa bez set_free */
    for (symbol_t *sym = s->scopes[s->top].head; sym; sym = sym->next) {
        if (sym->kind == SYM_VAR && sym->live) {
            diag(s, 0, "warning: variable left alive without set_free",
                 sym->name, true);
        }
    }

    if (s->top > 0) {
        s->top--;
    }
}

static symbol_t *declare(sema_ctx_t *s, const char *name, uint8_t kind,
                         cl_type_t type, uint32_t line)
{
    for (symbol_t *sym = s->scopes[s->top].head; sym; sym = sym->next) {
        bool same = true;

        for (int i = 0; name[i] || sym->name[i]; i++) {
            if (name[i] != sym->name[i]) {
                same = false;
                break;
            }
        }

        if (same) {
            diag(s, line, "redeclaration in same scope", name, false);
            return sym;
        }
    }

    symbol_t *sym = (symbol_t *)cl_arena_alloc(s->ar, sizeof(symbol_t));

    if (sym == (void *)0) {
        return (void *)0;
    }

    for (int i = 0; name[i] && i < 63; i++) {
        sym->name[i] = name[i];
        sym->name[i + 1] = '\0';
    }

    sym->kind = kind;
    sym->type = type;
    sym->live = true;
    sym->pool_ref = false;
    sym->freed = false;
    sym->value = 0;
    sym->depth = s->top;
    sym->nparams = 0;
    sym->node = (void *)0;
    sym->next = s->scopes[s->top].head;
    s->scopes[s->top].head = sym;

    return sym;
}

static symbol_t *lookup(sema_ctx_t *s, const char *name)
{
    for (int i = s->top; i >= 0; i--) {
        for (symbol_t *sym = s->scopes[i].head; sym; sym = sym->next) {
            bool same = true;

            for (int k = 0; name[k] || sym->name[k]; k++) {
                if (name[k] != sym->name[k]) {
                    same = false;
                    break;
                }
            }

            if (same) {
                return sym;
            }
        }
    }

    return (void *)0;
}

/* ---- zakresy wartości ---- */

static bool value_fits(uint64_t v, bool negative, cl_type_t t)
{
    if (t.bits == 0) {
        return true;
    }

    if (t.bits >= 64) {
        return true;
    }

    if (negative) {
        if (!t.neg) {
            return false;
        }

        return v <= (1ULL << (t.bits - 1));
    }

    if (t.neg) {
        return v <= (1ULL << (t.bits - 1)) - 1;
    }

    return v < (1ULL << t.bits);
}

typedef struct {
    cl_type_t t;
    bool is_const;
    uint64_t cval;
    bool cneg;
} etype_t;

static etype_t check_expr(sema_ctx_t *s, ast_node_t *n);

static void shared_note(sema_ctx_t *s, const char *name, cl_type_t t,
                        uint32_t line)
{
    if (s->sh_count < 2) {
        for (int i = 0; name[i] && i < 63; i++) {
            s->sh_name[s->sh_count][i] = name[i];
            s->sh_name[s->sh_count][i + 1] = '\0';
        }

        s->sh_type[s->sh_count] = t;
        s->sh_line[s->sh_count] = line;
        s->sh_count++;
    }
}

static void shared_validate(sema_ctx_t *s)
{
    if (s->sh_count == 1) {
        diag(s, s->sh_line[0], "$@ requires two participating variables",
             s->sh_name[0], false);
    } else if (s->sh_count == 2) {
        if (s->sh_type[0].bits != s->sh_type[1].bits) {
            diag(s, s->sh_line[1], "$@ pair must share the same range",
                 s->sh_name[1], false);
        }
    }

    s->sh_count = 0;
}

static etype_t check_expr(sema_ctx_t *s, ast_node_t *n)
{
    etype_t r = { { 0, false }, false, 0, false };

    if (n == (void *)0) {
        return r;
    }

    switch (n->kind) {
    case ND_NUM:
        r.is_const = true;
        r.cval = n->value;
        return r;

    case ND_CHAR:
        r.is_const = true;
        r.cval = n->value;
        r.t.bits = 64;
        return r;

    case ND_STR:
        r.t.bits = 64;   /* str = 64-bit */
        return r;

    case ND_UN:
        r = check_expr(s, n->a);

        if (n->op == 45 /* '-' */) {
            if (r.is_const) {
                r.cneg = true;
            } else if (r.t.bits != 0 && !r.t.neg) {
                diag(s, n->line, "negation needs _=> range", (void *)0, false);
            }
        }

        return r;

    case ND_VAR: {
        symbol_t *sym = lookup(s, n->name);

        if (sym == (void *)0) {
            diag(s, n->line, "undeclared identifier", n->name, false);
            return r;
        }

        if (sym->kind == SYM_VAR || sym->kind == SYM_STATIC) {
            if (!sym->live) {
                diag(s, n->line, "use after set_free", n->name, false);
            }

            r.t = sym->type;
        } else if (sym->kind == SYM_ENUM) {
            r.t.bits = 64;
        } else {
            diag(s, n->line, "not a value", n->name, false);
        }

        return r;
    }

    case ND_POOL: {
        symbol_t *sym = lookup(s, n->name);

        if (sym == (void *)0) {
            diag(s, n->line, "undeclared identifier", n->name, false);
            return r;
        }

        if (sym->depth != 0 || sym->kind != SYM_VAR) {
            diag(s, n->line, "$ works only on base pool variables",
                 n->name, false);
        }

        if (!sym->live) {
            diag(s, n->line, "use after set_free", n->name, false);
        }

        sym->pool_ref = true;

        if (n->pool_kind == 1 && !sym->type.neg) {
            diag(s, n->line, "$! needs neg range (_=>-1)", n->name, false);
        }

        if (n->pool_kind == 2) {
            sym->live = true;
            shared_note(s, n->name, sym->type, n->line);
        }

        r.t = sym->type;
        return r;
    }

    case ND_CALLIDX: {
        symbol_t *sym = lookup(s, n->name);

        if (sym == (void *)0) {
            diag(s, n->line, "call[] on undeclared", n->name, false);
            return r;
        }

        if ((sym->kind == SYM_VAR || sym->kind == SYM_STATIC) && !sym->live) {
            diag(s, n->line, "use after set_free", n->name, false);
        }

        r.t = (sym->kind == SYM_FN || sym->kind == SYM_EXTERN)
              ? (cl_type_t){ 64, false } : sym->type;
        return r;
    }

    case ND_SCOPECALL: {
        symbol_t *sc = lookup(s, n->name);

        if (sc == (void *)0) {
            diag(s, n->line, "unknown scope", n->name, false);
            return r;
        }

        if (sc->kind == SYM_ENUM) {
            bool found = false;

            if (sc->node != (void *)0) {
                for (size_t i = 0; i < sc->node->nlist; i++) {
                    ast_node_t *it = sc->node->list[i];

                    bool same = true;

                    for (int k = 0; it->name[k] || n->name2[k]; k++) {
                        if (it->name[k] != n->name2[k]) {
                            same = false;
                            break;
                        }
                    }

                    if (same) {
                        found = true;
                        r.is_const = true;
                        r.cval = it->value;
                        r.t.bits = 64;
                    }
                }
            }

            if (!found) {
                diag(s, n->line, "no such enum item", n->name2, false);
            }

            return r;
        }

        if (sc->kind == SYM_DATA) {
            bool found = false;

            if (sc->node != (void *)0 && sc->node->a != (void *)0) {
                ast_node_t *base = sc->node->a;

                for (size_t i = 0; i < base->nlist; i++) {
                    ast_node_t *e = base->list[i];

                    if (e->kind == ND_VAR) {
                        bool same = true;

                        for (int k = 0; e->name[k] || n->name2[k]; k++) {
                            if (e->name[k] != n->name2[k]) {
                                same = false;
                                break;
                            }
                        }

                        if (same) {
                            found = true;
                            r.t.bits = 64;
                        }
                    }
                }
            }

            if (!found) {
                diag(s, n->line, "no such data entry", n->name2, false);
            }

            return r;
        }

        diag(s, n->line, "scope is not enum/data", n->name, false);
        return r;
    }

    case ND_CALL: {
        if (n->a == (void *)0 || n->a->kind != ND_VAR) {
            diag(s, n->line, "bad call target", (void *)0, false);
            return r;
        }

        symbol_t *fn = lookup(s, n->a->name);

        if (fn == (void *)0) {
            diag(s, n->line, "call of undeclared", n->a->name, false);
            return r;
        }

        if (fn->kind == SYM_FN) {
            if (fn->nparams != n->nlist) {
                diag(s, n->line, "arity mismatch", fn->name, false);
            }

            for (size_t i = 0; i < n->nlist && i < fn->nparams; i++) {
                etype_t a = check_expr(s, n->list[i]);

                if (a.is_const &&
                    !value_fits(a.cval, a.cneg, fn->params[i])) {
                    diag(s, n->line, "arg out of range", fn->name, false);
                }
            }

            r.t.bits = 64;
            return r;
        }

        if (fn->kind == SYM_EXTERN) {
            for (size_t i = 0; i < n->nlist; i++) {
                check_expr(s, n->list[i]);
            }

            r.t.bits = 64;
            return r;
        }

        diag(s, n->line, "not callable", n->a->name, false);
        return r;
    }

    case ND_MEMBER:
        r = check_expr(s, n->a);
        return r;

    case ND_ASSIGN: {
        etype_t l = check_expr(s, n->a);
        etype_t rhs = check_expr(s, n->b);

        if (l.t.bits != 0) {
            if (rhs.is_const) {
                if (!value_fits(rhs.cval, rhs.cneg, l.t)) {
                    diag(s, n->line, "value out of range", (void *)0, false);
                }
            } else if (rhs.t.bits != 0) {
                if (rhs.t.bits > l.t.bits) {
                    diag(s, n->line, "narrowing assignment", (void *)0, false);
                } else if (rhs.t.bits == l.t.bits &&
                           rhs.t.neg != l.t.neg) {
                    diag(s, n->line, "signed/unsigned mismatch",
                         (void *)0, false);
                }
            }
        }

        return l;
    }

    case ND_BIN: {
        etype_t a = check_expr(s, n->a);
        etype_t b = check_expr(s, n->b);

        bool cmp = n->op == TK_EQ || n->op == TK_NEQ || n->op == TK_LT ||
                   n->op == TK_GT || n->op == TK_LEQ || n->op == TK_GEQ;

        if (cmp) {
            r.t.bits = 1;
            return r;
        }

        if (a.t.bits == 0 && b.t.bits != 0) {
            if (a.is_const && !value_fits(a.cval, a.cneg, b.t)) {
                diag(s, n->line, "value out of range", (void *)0, false);
            }

            r.t = b.t;
        } else if (b.t.bits == 0 && a.t.bits != 0) {
            if (b.is_const && !value_fits(b.cval, b.cneg, a.t)) {
                diag(s, n->line, "value out of range", (void *)0, false);
            }

            r.t = a.t;
        } else {
            r.t.bits = a.t.bits > b.t.bits ? a.t.bits : b.t.bits;
            r.t.neg = a.t.neg || b.t.neg;
        }

        return r;
    }

    default:
        return r;
    }
}

/* ---- statementy ---- */

static void check_block(sema_ctx_t *s, ast_node_t *n);

static void check_stmt(sema_ctx_t *s, ast_node_t *n)
{
    s->sh_count = 0;

    switch (n->kind) {
    case ND_LET: {
        etype_t init = check_expr(s, n->a);

        if (init.is_const &&
            !value_fits(init.cval, init.cneg, n->type)) {
            diag(s, n->line, "literal out of type range", n->name, false);
        }

        if (init.t.bits != 0 && init.t.bits > n->type.bits) {
            diag(s, n->line, "narrowing assignment", n->name, false);
        }

        symbol_t *sym = declare(s, n->name, SYM_VAR, n->type, n->line);

        if (sym != (void *)0 && n->type.neg) {
            sym->type.neg = true;
        }

        break;
    }

    case ND_STATIC:
        declare(s, "data", SYM_STATIC, n->type, n->line);
        break;

    case ND_SET_FREE: {
        symbol_t *sym = lookup(s, n->name);

        if (sym == (void *)0) {
            diag(s, n->line, "set_free on undeclared", n->name, false);
        } else if (sym->freed) {
            diag(s, n->line, "double set_free", n->name, false);
        } else {
            sym->live = false;
            sym->freed = true;
        }

        break;
    }

    case ND_IF:
        if (n->a != (void *)0) {
            check_expr(s, n->a);
        }

        check_block(s, n->b);

        if (n->c != (void *)0) {
            if (n->c->kind == ND_BLOCK) {
                check_block(s, n->c);
            } else {
                check_stmt(s, n->c);
            }
        }

        break;

    case ND_WHEN: {
        symbol_t *tgt = lookup(s, n->name);

        if (tgt == (void *)0) {
            diag(s, n->line, "when on undeclared", n->name, false);
        }

        for (size_t i = 0; i < n->nlist; i++) {
            ast_node_t *c = n->list[i];

            if (c->a != (void *)0 && c->a->kind == ND_BIN &&
                c->a->a != (void *)0 && c->a->a->kind == ND_MEMBER &&
                c->a->a->a != (void *)0 &&
                c->a->a->a->kind == ND_VAR) {
                const char *cn = c->a->a->a->name;
                bool same = true;

                for (int k = 0; cn[k] || n->name[k]; k++) {
                    if (cn[k] != n->name[k]) {
                        same = false;
                        break;
                    }
                }

                if (!same) {
                    diag(s, c->line, "case target mismatch", cn, false);
                }

                if (tgt != (void *)0 && c->a->b != (void *)0 &&
                    c->a->b->kind == ND_NUM &&
                    !value_fits(c->a->b->value, false, tgt->type)) {
                    diag(s, c->line, "case value out of range",
                         n->name, false);
                }
            }

            check_block(s, c->b);
        }

        break;
    }

    case ND_BLOCK:
        check_block(s, n);
        break;

    default:
        check_expr(s, n);
        shared_validate(s);
        break;
    }

    if (n->kind != ND_BLOCK) {
        shared_validate(s);
    }
}

static void check_block(sema_ctx_t *s, ast_node_t *n)
{
    push_scope(s);

    for (size_t i = 0; i < n->nlist; i++) {
        check_stmt(s, n->list[i]);
    }

    pop_scope(s);
}

/* ---- top level ---- */

static void declare_top(sema_ctx_t *s, ast_node_t *n)
{
    switch (n->kind) {
    case ND_FN: {
        cl_type_t t = { 0, false };
        symbol_t *sym = declare(s, n->name, SYM_FN, t, n->line);

        if (sym == (void *)0) {
            return;
        }

        for (size_t i = 0; i < n->nlist && i < 8; i++) {
            sym->params[sym->nparams++] = n->list[i]->type;
        }

        sym->node = n;
        break;
    }

    case ND_LET:
        declare(s, n->name, SYM_VAR, n->type, n->line);
        break;

    case ND_STATIC:
        declare(s, "data", SYM_STATIC, n->type, n->line);
        break;

    case ND_ENUM: {
        cl_type_t t = { 64, false };
        symbol_t *sym = declare(s, n->name, SYM_ENUM, t, n->line);

        if (sym != (void *)0) {
            sym->node = n;
        }

        break;
    }

    case ND_EXTERN:
        for (size_t i = 0; i < n->nlist; i++) {
            cl_type_t t = { 64, false };
            declare(s, n->list[i]->name, SYM_EXTERN, t, n->line);
        }

        break;

    case ND_DATAIMPL: {
        cl_type_t t = { 0, false };
        symbol_t *sym = declare(s, "data", SYM_DATA, t, n->line);

        if (sym != (void *)0) {
            sym->node = n;
        }

        break;
    }

    default:
        break;
    }
}

int cl_sema_run(sema_ctx_t *s, ast_node_t *prog)
{
    if (prog == (void *)0) {
        return -1;
    }

    /* pass 1: deklaracje top-level */
    for (size_t i = 0; i < prog->nlist; i++) {
        declare_top(s, prog->list[i]);
    }

    /* pass 2: ciała funkcji */
    for (size_t i = 0; i < prog->nlist; i++) {
        ast_node_t *n = prog->list[i];

        if (n->kind != ND_FN) {
            continue;
        }

        push_scope(s);

        for (size_t k = 0; k < n->nlist; k++) {
            declare(s, n->list[k]->name, SYM_VAR, n->list[k]->type, n->line);
        }

        check_block(s, n->a);

        pop_scope(s);
    }

    return (int)s->ndiags;
}

size_t cl_sema_diag_count(const sema_ctx_t *s)
{
    return s->ndiags;
}

const cl_diag_t *cl_sema_diag(const sema_ctx_t *s, size_t i)
{
    return i < s->ndiags ? &s->diags[i] : (void *)0;
}