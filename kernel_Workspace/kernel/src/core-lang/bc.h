#ifndef CORELANG_BC_H
#define CORELANG_BC_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "ast.h"

typedef struct {
    uint8_t b[64];   
} cl_val_t;

enum {
    OP_NOP = 0,
    OP_CONST,
    OP_LOAD, OP_STORE,
    OP_GLOAD, OP_GSTORE,
    OP_ADD, OP_SUB, OP_MUL, OP_DIV, OP_MOD,
    OP_AND, OP_OR, OP_XOR, OP_SHL, OP_SHR,
    OP_NEG, OP_NOT,
    OP_EQ, OP_NE, OP_LT, OP_GT, OP_LE, OP_GE,
    OP_JMP, OP_JZ, OP_JNZ,
    OP_CALL, OP_RET, OP_EXTERN,
    OP_POP,
    OP_FREE, OP_SHARED,
    OP_HALT,
};

typedef struct {
    uint8_t op;
    uint8_t bits;    
    uint16_t a, b;
    uint32_t c;
} cl_insn_t;

typedef struct {
    uint32_t start;
    uint16_t nslots;
    uint16_t nargs;
} cl_fn_t;

typedef struct {
    cl_insn_t *code;
    size_t code_len, code_cap;

    cl_val_t consts[64];
    uint8_t const_bits[64];
    size_t nconsts;

    cl_fn_t fns[16];
    char fn_names[16][64];
    size_t nfns;
    int main_fn;

    char ext_names[16][64];
    size_t nexts;

    char gnames[64][64];
    uint8_t gbits[64];
    uint8_t gneg[64];
    cl_val_t ginit[64];
    size_t nglobals;

    arena_t *ar;
} cl_prog_t;

#endif