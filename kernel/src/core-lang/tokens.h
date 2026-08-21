#ifndef CORELANG_TOKENS_H
#define CORELANG_TOKENS_H

#include <stdint.h>

typedef enum {
    TK_EOF = 0,
    TK_ERR,

    TK_IDENT,
    TK_TYPE,      /* u4..u512, str (width w type_bits) */
    TK_NUM,
    TK_STR,
    TK_CHAR,

    /* keywords */
    TK_IF, TK_ELSE, TK_WHEN, TK_CASE,
    TK_FN, TK_LET, TK_STATIC, TK_DATA, TK_DATAIMPL,
    TK_ENUM, TK_EXTERN, TK_CALL, TK_SET_FREE,

    /* symbols */
    TK_SEMI,        /* ; */
    TK_COMMA,       /* , */
    TK_DOT,         /* . */
    TK_COLON,       /* : */
    TK_DCOLON,      /* :: */
    TK_ASSIGN,      /* = */
    TK_EQCOLON,     /* =: */
    TK_ARROW,       /* => */
    TK_EQ,          /* == */
    TK_NEQ,         /* != */
    TK_LT, TK_GT, TK_LEQ, TK_GEQ,
    TK_PLUS, TK_MINUS, TK_STAR, TK_SLASH, TK_PERCENT,
    TK_AMP, TK_PIPE, TK_CARET, TK_TILDE, TK_BANG,
    TK_SHL, TK_SHR,
    TK_LBRACE, TK_RBRACE,
    TK_LBRACK, TK_RBRACK,
    TK_LPAREN, TK_RPAREN,
    TK_UNDERSCORE,  /* _ */

    /* pula bazowa */
    TK_DOLLAR,       /* $x  */
    TK_DOLLAR_BANG,  /* $!x */
    TK_DOLLAR_AT,    /* $@x */
} tok_kind_t;

typedef struct {
    tok_kind_t kind;
    const char *start;   /* wskazuje w źródle, nie kopiuje */
    uint32_t len;
    uint64_t num;        /* TK_NUM / TK_CHAR */
    uint16_t type_bits;  /* TK_TYPE: 4..512, str=64 */
    uint32_t line;
    uint32_t col;
} token_t;

const char *cl_tok_name(tok_kind_t kind);

#endif