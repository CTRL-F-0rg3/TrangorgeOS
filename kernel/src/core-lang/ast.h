#ifndef CORELANG_AST_H
#define CORELANG_AST_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "tokens.h"   /* <- DODAJ: op w node'ach to tok_kind_t */
typedef struct cl_type {
    uint16_t bits;   /* 4..512, str=64 */
    bool neg;        /* _=>-1 : zakres obejmuje ujemne */
} cl_type_t;

enum {
    ND_PROGRAM = 0,
    ND_FN, ND_LET, ND_STATIC, ND_ENUM, ND_EXTERN, ND_DATAIMPL,
    ND_BLOCK, ND_IF, ND_WHEN, ND_CASE, ND_SET_FREE,
    ND_ASSIGN, ND_BIN, ND_UN, ND_MEMBER,
    ND_VAR, ND_POOL, ND_SCOPECALL, ND_CALL, ND_CALLIDX,
    ND_NUM, ND_STR, ND_CHAR,
};

typedef struct ast_node ast_node_t;

struct ast_node {
    uint8_t kind;
    cl_type_t type;
    uint32_t line;

    int op;             /* token kind dla BIN/UN */
    int pool_kind;      /* 0=$ 1=$! 2=$@ */
    bool is_array;
    uint64_t value;
    uint64_t arr_count, arr_init;

    char name[64];
    char name2[64];

    ast_node_t *a, *b, *c;
    ast_node_t **list;
    size_t nlist;
};

typedef struct {
    uint8_t *buf;
    size_t cap;
    size_t pos;
} arena_t;

void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap);
void *cl_arena_alloc(arena_t *a, size_t n);
ast_node_t *cl_node(arena_t *ar, uint8_t kind, uint32_t line);

#endif