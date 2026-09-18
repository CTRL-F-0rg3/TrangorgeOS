#ifndef CORELANG_PARSER_H
#define CORELANG_PARSER_H

#include "tokens.h"
#include "ast.h"

typedef struct {
    token_t *toks;
    size_t n;
    size_t pos;
    arena_t *ar;
    bool failed;
    const char *err;
    uint32_t err_line;
} parser_t;


ast_node_t *cl_parse(parser_t *p, token_t *toks, size_t n, arena_t *ar);

#endif