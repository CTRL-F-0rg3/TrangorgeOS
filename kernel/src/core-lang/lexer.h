#ifndef CORELANG_LEXER_H
#define CORELANG_LEXER_H

#include "tokens.h"
#include <stddef.h>
#include <stdbool.h>

typedef struct {
    const char *src;
    size_t len;
    size_t pos;
    uint32_t line;
    uint32_t col;
} lexer_t;

void cl_lexer_init(lexer_t *l, const char *src, size_t len);

bool cl_lex_next(lexer_t *l, token_t *out);

int cl_lex_all(lexer_t *l, token_t *buf, size_t cap, size_t *count);

#endif