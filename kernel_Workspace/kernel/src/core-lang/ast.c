#include "ast.h"

void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap)
{
    a->buf = buf;
    a->cap = cap;
    a->pos = 0;
}

void *cl_arena_alloc(arena_t *a, size_t n)
{
    size_t aligned = (n + 15) & ~(size_t)15;

    if (a->pos + aligned > a->cap) {
        return (void *)0;
    }

    void *p = a->buf + a->pos;

    a->pos += aligned;

    return p;
}

ast_node_t *cl_node(arena_t *ar, uint8_t kind, uint32_t line)
{
    ast_node_t *n = (ast_node_t *)cl_arena_alloc(ar, sizeof(ast_node_t));

    if (n == (void *)0) {
        return (void *)0;
    }

    n->kind = kind;
    n->type.bits = 0;
    n->type.neg = false;
    n->line = line;
    n->op = 0;
    n->pool_kind = 0;
    n->is_array = false;
    n->value = 0;
    n->arr_count = 0;
    n->arr_init = 0;
    n->name[0] = '\0';
    n->name2[0] = '\0';
    n->a = n->b = n->c = (void *)0;
    n->list = (void *)0;
    n->nlist = 0;

    return n;
}