#ifndef CORELANG_SEMA_H
#define CORELANG_SEMA_H

#include "ast.h"

typedef struct {
    uint32_t line;
    const char *msg;
    char name[64];
    bool is_warn;
} cl_diag_t;

typedef struct sema_ctx sema_ctx_t;

sema_ctx_t *cl_sema_new(arena_t *ar);
int cl_sema_run(sema_ctx_t *s, ast_node_t *prog);

size_t cl_sema_diag_count(const sema_ctx_t *s);
const cl_diag_t *cl_sema_diag(const sema_ctx_t *s, size_t i);

#endif