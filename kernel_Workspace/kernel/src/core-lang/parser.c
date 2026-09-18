#include "parser.h"

static void copy_name(char *dst, size_t cap, const token_t *t)
{
    size_t n = t->len;

    if (n >= cap) {
        n = cap - 1;
    }

    for (size_t i = 0; i < n; i++) {
        dst[i] = t->start[i];
    }

    dst[n] = '\0';
}

static token_t *peek(parser_t *p)
{
    return &p->toks[p->pos];
}

static token_t *peek_at(parser_t *p, size_t off)
{
    size_t i = p->pos + off;

    if (i >= p->n) {
        i = p->n - 1;
    }

    return &p->toks[i];
}

static token_t *advance(parser_t *p)
{
    token_t *t = &p->toks[p->pos];

    if (p->pos + 1 < p->n) {
        p->pos++;
    }

    return t;
}

static bool check(parser_t *p, tok_kind_t k)
{
    return peek(p)->kind == k;
}

static bool match(parser_t *p, tok_kind_t k)
{
    if (check(p, k)) {
        advance(p);
        return true;
    }

    return false;
}

static void perr(parser_t *p, const char *m)
{
    if (!p->failed) {
        p->failed = true;
        p->err = m;
        p->err_line = peek(p)->line;
    }
}

static token_t *expect(parser_t *p, tok_kind_t k, const char *m)
{
    if (!check(p, k)) {
        perr(p, m);
        return peek(p);
    }

    return advance(p);
}

static ast_node_t *new_node(parser_t *p, uint8_t kind)
{
    return cl_node(p->ar, kind, peek(p)->line);
}

static ast_node_t *parse_expr(parser_t *p);
static ast_node_t *parse_block(parser_t *p);

/* ---- lista node'ów (rośnie przez arena-realloc nie jest możliwy,
        więc zbieramy do tymczasowej tablicy na stacku i kopiujemy) ---- */

#define LIST_MAX 128

typedef struct {
    ast_node_t *v[LIST_MAX];
    size_t n;
} nlist_t;

static void nl_push(parser_t *p, nlist_t *l, ast_node_t *x)
{
    if (l->n < LIST_MAX) {
        l->v[l->n++] = x;
    } else {
        perr(p, "list too long");
    }
}

static void nl_commit(parser_t *p, nlist_t *l, ast_node_t *parent)
{
    ast_node_t **arr = (ast_node_t **)cl_arena_alloc(
        p->ar, l->n * sizeof(ast_node_t *));

    if (arr == (void *)0) {
        perr(p, "arena full");
        return;
    }

    for (size_t i = 0; i < l->n; i++) {
        arr[i] = l->v[i];
    }

    parent->list = arr;
    parent->nlist = l->n;
}

/* ---- typy ---- */

static bool parse_type(parser_t *p, cl_type_t *out)
{
    if (check(p, TK_TYPE)) {
        out->bits = peek(p)->type_bits;
        out->neg = false;
        advance(p);
        return true;
    }

    perr(p, "expected type (u4..u512/str)");
    return false;
}

/* ---- deklaracje ---- */

static ast_node_t *parse_let(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_LET);

    expect(p, TK_LET, "let");

    parse_type(p, &n->type);

    copy_name(n->name, sizeof(n->name), expect(p, TK_IDENT, "let name"));

    expect(p, TK_ASSIGN, "let =");

    n->a = parse_expr(p);

    expect(p, TK_SEMI, "let ;");

    /* opcjonalny zakres ujemny: _=>-1; */
    if (match(p, TK_UNDERSCORE)) {
        expect(p, TK_ARROW, "_=>");
        ast_node_t *r = parse_expr(p);
        expect(p, TK_SEMI, "range ;");

        n->type.neg = true;
        n->b = r;
    }

    return n;
}

static ast_node_t *parse_static(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_STATIC);

    expect(p, TK_STATIC, "static");
    expect(p, TK_DATA, "static data");
    expect(p, TK_LBRACK, "[");

    parse_type(p, &n->type);

    expect(p, TK_COLON, ":");

    n->arr_count = expect(p, TK_NUM, "count")->num;

    expect(p, TK_SEMI, ";");

    n->arr_init = expect(p, TK_NUM, "init")->num;

    expect(p, TK_RBRACK, "]");
    expect(p, TK_SEMI, "static ;");

    n->is_array = true;

    return n;
}

static ast_node_t *parse_fn(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_FN);

    expect(p, TK_FN, "fn");

    copy_name(n->name, sizeof(n->name), expect(p, TK_IDENT, "fn name"));

    expect(p, TK_LPAREN, "fn (");

    nlist_t params = { { 0 }, 0 };

    while (!check(p, TK_RPAREN) && !check(p, TK_EOF)) {
        ast_node_t *arg = new_node(p, ND_LET);

        parse_type(p, &arg->type);
        copy_name(arg->name, sizeof(arg->name),
                  expect(p, TK_IDENT, "param name"));

        nl_push(p, &params, arg);

        if (!match(p, TK_COMMA)) {
            break;
        }
    }

    expect(p, TK_RPAREN, "fn )");

    nl_commit(p, &params, n);

    n->a = parse_block(p);

    return n;
}

static ast_node_t *parse_enum(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_ENUM);

    expect(p, TK_ENUM, "enum");

    copy_name(n->name, sizeof(n->name), expect(p, TK_IDENT, "enum name"));

    expect(p, TK_LBRACE, "enum {");

    nlist_t items = { { 0 }, 0 };
    uint64_t val = 0;

    while (!check(p, TK_RBRACE) && !check(p, TK_EOF)) {
        ast_node_t *it = new_node(p, ND_VAR);

        copy_name(it->name, sizeof(it->name),
                  expect(p, TK_IDENT, "enum item"));

        it->value = val++;

        nl_push(p, &items, it);

        if (!match(p, TK_COMMA)) {
            break;
        }
    }

    expect(p, TK_RBRACE, "enum }");

    nl_commit(p, &items, n);

    return n;
}

static ast_node_t *parse_extern(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_EXTERN);

    expect(p, TK_EXTERN, "extern");

    token_t *abi = expect(p, TK_CHAR, "extern 'C'");

    n->name[0] = (char)abi->num;
    n->name[1] = '\0';

    expect(p, TK_LBRACE, "extern {");

    nlist_t fns = { { 0 }, 0 };

    while (!check(p, TK_RBRACE) && !check(p, TK_EOF)) {
        ast_node_t *f = new_node(p, ND_VAR);

        copy_name(f->name, sizeof(f->name),
                  expect(p, TK_IDENT, "extern fn"));

        nl_push(p, &fns, f);

        if (!match(p, TK_COMMA)) {
            break;
        }
    }

    expect(p, TK_RBRACE, "extern }");

    nl_commit(p, &fns, n);

    return n;
}

static ast_node_t *parse_dataimpl(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_DATAIMPL);

    expect(p, TK_DATAIMPL, "dataimpl");

    expect(p, TK_LBRACE, "dataimpl {");

    /* baza: data: ... aż do } */
    nlist_t base = { { 0 }, 0 };

    while (!check(p, TK_RBRACE) && !check(p, TK_EOF)) {
        if (match(p, TK_DATA)) {
            expect(p, TK_COLON, "data:");
            continue;
        }

        nl_push(p, &base, parse_expr(p));

        if (!match(p, TK_COMMA)) {
            break;
        }
    }

    expect(p, TK_RBRACE, "dataimpl }");

    expect(p, TK_ARROW, "=>");

    ast_node_t *baseblk = new_node(p, ND_BLOCK);
    nl_commit(p, &base, baseblk);

    n->a = baseblk;
    n->b = parse_block(p);

    return n;
}

/* ---- statementy ---- */

static ast_node_t *parse_block(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_BLOCK);

    expect(p, TK_LBRACE, "{");

    nlist_t stmts = { { 0 }, 0 };

    while (!check(p, TK_RBRACE) && !check(p, TK_EOF)) {
        ast_node_t *s = (void *)0;

        switch (peek(p)->kind) {
        case TK_LET:      s = parse_let(p); break;
        case TK_STATIC:   s = parse_static(p); break;
        case TK_IF:       s = (void *)1; break; /* niżej */
        case TK_WHEN:     s = (void *)1; break;
        case TK_SET_FREE: {
            s = new_node(p, ND_SET_FREE);
            advance(p);
            expect(p, TK_LBRACE, "set_free {");
            copy_name(s->name, sizeof(s->name),
                      expect(p, TK_IDENT, "set_free var"));
            expect(p, TK_RBRACE, "set_free }");
            expect(p, TK_SEMI, "set_free ;");
            break;
        }
        default:
            s = (void *)1;
            break;
        }

        if (s == (void *)1) {
            if (check(p, TK_IF)) {
                extern ast_node_t *cl_parse_if(parser_t *p);
                s = cl_parse_if(p);
            } else if (check(p, TK_WHEN)) {
                extern ast_node_t *cl_parse_when(parser_t *p);
                s = cl_parse_when(p);
            } else {
                s = parse_expr(p);
                expect(p, TK_SEMI, "stmt ;");
            }
        }

        nl_push(p, &stmts, s);
    }

    expect(p, TK_RBRACE, "}");

    nl_commit(p, &stmts, n);

    return n;
}

ast_node_t *cl_parse_if(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_IF);

    expect(p, TK_IF, "if");

    if (!check(p, TK_LBRACE)) {
        n->a = parse_expr(p);
    }

    n->b = parse_block(p);

    /* else / else if / if else */
    if (match(p, TK_ELSE)) {
        if (check(p, TK_IF) && peek_at(p, 1)->kind != TK_ELSE) {
            advance(p);

            ast_node_t *inner = new_node(p, ND_IF);

            if (!check(p, TK_LBRACE)) {
                inner->a = parse_expr(p);
            }

            inner->b = parse_block(p);
            n->c = inner;
        } else {
            n->c = parse_block(p);
        }
    } else if (check(p, TK_IF) && peek_at(p, 1)->kind == TK_ELSE) {
        advance(p);
        advance(p);
        n->c = parse_block(p);
    }

    return n;
}

ast_node_t *cl_parse_when(parser_t *p)
{
    ast_node_t *n = new_node(p, ND_WHEN);

    expect(p, TK_WHEN, "when");

    copy_name(n->name, sizeof(n->name), expect(p, TK_IDENT, "when target"));

    expect(p, TK_EQCOLON, "=:");
    expect(p, TK_LBRACE, "when {");

    nlist_t cases = { { 0 }, 0 };

    while (!check(p, TK_RBRACE) && !check(p, TK_EOF)) {
        ast_node_t *c = new_node(p, ND_CASE);

        c->a = parse_expr(p);
        c->b = parse_block(p);

        nl_push(p, &cases, c);
    }

    expect(p, TK_RBRACE, "when }");

    nl_commit(p, &cases, n);

    return n;
}

/* ---- wyrażenia ---- */

static ast_node_t *parse_primary(parser_t *p)
{
    ast_node_t *n;

    if (check(p, TK_NUM)) {
        n = new_node(p, ND_NUM);
        n->value = advance(p)->num;
        return n;
    }

    if (check(p, TK_STR)) {
        n = new_node(p, ND_STR);
        token_t *t = advance(p);
        copy_name(n->name, sizeof(n->name), t);
        return n;
    }

    if (check(p, TK_CHAR)) {
        n = new_node(p, ND_CHAR);
        n->value = advance(p)->num;
        return n;
    }

    if (check(p, TK_DOLLAR) || check(p, TK_DOLLAR_BANG) ||
        check(p, TK_DOLLAR_AT)) {
        n = new_node(p, ND_POOL);
        token_t *t = advance(p);

        n->pool_kind = (t->kind == TK_DOLLAR) ? 0
                     : (t->kind == TK_DOLLAR_BANG) ? 1 : 2;

        copy_name(n->name, sizeof(n->name),
                  expect(p, TK_IDENT, "$var"));
        return n;
    }

    if (check(p, TK_CALL) && peek_at(p, 1)->kind == TK_LBRACK) {
        n = new_node(p, ND_CALLIDX);
        advance(p);
        expect(p, TK_LBRACK, "call[");
        copy_name(n->name, sizeof(n->name),
                  expect(p, TK_IDENT, "call[x]"));
        expect(p, TK_RBRACK, "]");
        return n;
    }

    if (match(p, TK_LPAREN)) {
        n = parse_expr(p);
        expect(p, TK_RPAREN, ")");
        return n;
    }

    if (check(p, TK_IDENT)) {
        n = new_node(p, ND_VAR);
        copy_name(n->name, sizeof(n->name), advance(p));
        return n;
    }

    perr(p, "unexpected token in expr");
    advance(p);

    return new_node(p, ND_NUM);
}

static ast_node_t *parse_postfix(parser_t *p)
{
    ast_node_t *n = parse_primary(p);

    for (;;) {
        if (match(p, TK_DOT)) {
            ast_node_t *m = new_node(p, ND_MEMBER);

            m->a = n;
            copy_name(m->name, sizeof(m->name),
                      expect(p, TK_IDENT, "member"));
            n = m;
        } else if (check(p, TK_DCOLON)) {
            advance(p);

            ast_node_t *m = new_node(p, ND_SCOPECALL);

            for (size_t i = 0; i < sizeof(m->name); i++) {
                m->name[i] = 0;
            }

            if (n->kind == ND_VAR) {
                for (size_t i = 0; n->name[i] && i < 63; i++) {
                    m->name[i] = n->name[i];
                }
            }

            copy_name(m->name2, sizeof(m->name2),
                      expect(p, TK_IDENT, "scope fn"));

            expect(p, TK_LPAREN, "scope (");

            nlist_t args = { { 0 }, 0 };

            while (!check(p, TK_RPAREN) && !check(p, TK_EOF)) {
                nl_push(p, &args, parse_expr(p));

                if (!match(p, TK_COMMA)) {
                    break;
                }
            }

            expect(p, TK_RPAREN, "::)");

            nl_commit(p, &args, m);

            n = m;
        } else if (check(p, TK_LPAREN)) {
            advance(p);

            ast_node_t *m = new_node(p, ND_CALL);

            m->a = n;

            nlist_t args = { { 0 }, 0 };

            while (!check(p, TK_RPAREN) && !check(p, TK_EOF)) {
                nl_push(p, &args, parse_expr(p));

                if (!match(p, TK_COMMA)) {
                    break;
                }
            }

            expect(p, TK_RPAREN, "call )");

            nl_commit(p, &args, m);

            n = m;
        } else {
            break;
        }
    }

    return n;
}

static ast_node_t *parse_unary(parser_t *p)
{
    if (check(p, TK_MINUS) || check(p, TK_BANG) ||
        check(p, TK_TILDE)) {
        ast_node_t *n = new_node(p, ND_UN);

        n->op = advance(p)->kind;
        n->a = parse_unary(p);

        return n;
    }

    return parse_postfix(p);
}

static ast_node_t *parse_bin(parser_t *p, int level)
{
    ast_node_t *lhs = parse_unary(p);

    for (;;) {
        tok_kind_t k = peek(p)->kind;
        int lv = 0;

        switch (k) {
        case TK_STAR: case TK_SLASH: case TK_PERCENT: lv = 1; break;
        case TK_PLUS: case TK_MINUS: lv = 2; break;
        case TK_SHL: case TK_SHR: lv = 3; break;
        case TK_LT: case TK_GT: case TK_LEQ: case TK_GEQ: lv = 4; break;
        case TK_EQ: case TK_NEQ: lv = 5; break;
        case TK_AMP: lv = 6; break;
        case TK_CARET: lv = 7; break;
        case TK_PIPE: lv = 8; break;
        default: return lhs;
        }

        if (lv > level) {
            return lhs;
        }

        advance(p);

        ast_node_t *rhs = parse_bin(p, lv + 1 > 8 ? 8 : lv + 1);

        ast_node_t *b = new_node(p, ND_BIN);

        b->op = k;
        b->a = lhs;
        b->b = rhs;

        lhs = b;

        if (k == TK_PIPE && level >= 8) return lhs;
    }
}

static ast_node_t *parse_assign(parser_t *p)
{
    ast_node_t *lhs = parse_bin(p, 1);

    if (match(p, TK_ASSIGN)) {
        ast_node_t *n = new_node(p, ND_ASSIGN);

        n->a = lhs;
        n->b = parse_assign(p);

        return n;
    }

    return lhs;
}

static ast_node_t *parse_expr(parser_t *p)
{
    return parse_assign(p);
}

/* ---- top ---- */

ast_node_t *cl_parse(parser_t *p, token_t *toks, size_t n, arena_t *ar)
{
    p->toks = toks;
    p->n = n;
    p->pos = 0;
    p->ar = ar;
    p->failed = false;
    p->err = (void *)0;

    ast_node_t *prog = cl_node(ar, ND_PROGRAM, 1);

    nlist_t tops = { { 0 }, 0 };

    while (!check(p, TK_EOF)) {
        ast_node_t *d;

        switch (peek(p)->kind) {
        case TK_FN:        d = parse_fn(p); break;
        case TK_LET:       d = parse_let(p); break;
        case TK_STATIC:    d = parse_static(p); break;
        case TK_ENUM:      d = parse_enum(p); break;
        case TK_EXTERN:    d = parse_extern(p); break;
        case TK_DATAIMPL:  d = parse_dataimpl(p); break;
        default:
            perr(p, "expected top-level decl");
            advance(p);
            continue;
        }

        nl_push(p, &tops, d);

        if (p->failed) {
            break;
        }
    }

    nl_commit(p, &tops, prog);

    if (p->failed) {
        return (void *)0;
    }

    return prog;
}