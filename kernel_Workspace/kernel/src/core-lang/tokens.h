#ifndef CORELANG_TOKENS_H
#define CORELANG_TOKENS_H

#include <stdint.h>

typedef enum {
    TK_EOF = 0,
    TK_ERR,

    TK_IDENT,
    TK_TYPE,      
    TK_NUM,
    TK_STR,
    TK_CHAR,


    TK_IF, TK_ELSE, TK_WHEN, TK_CASE,
    TK_FN, TK_LET, TK_STATIC, TK_DATA, TK_DATAIMPL,
    TK_ENUM, TK_EXTERN, TK_CALL, TK_SET_FREE,

    TK_SEMI,        
    TK_COMMA,      
    TK_DOT,        
    TK_COLON,       
    TK_DCOLON,      
    TK_ASSIGN,     
    TK_EQCOLON,     
    TK_ARROW,       
    TK_EQ,          
    TK_NEQ,         
    TK_LT, TK_GT, TK_LEQ, TK_GEQ,
    TK_PLUS, TK_MINUS, TK_STAR, TK_SLASH, TK_PERCENT,
    TK_AMP, TK_PIPE, TK_CARET, TK_TILDE, TK_BANG,
    TK_SHL, TK_SHR,
    TK_LBRACE, TK_RBRACE,
    TK_LBRACK, TK_RBRACK,
    TK_LPAREN, TK_RPAREN,
    TK_UNDERSCORE,  


    TK_DOLLAR,       
    TK_DOLLAR_BANG,  
    TK_DOLLAR_AT,    
} tok_kind_t;

typedef struct {
    tok_kind_t kind;
    const char *start;   
    uint32_t len;
    uint64_t num;        
    uint16_t type_bits;  
    uint32_t line;
    uint32_t col;
} token_t;

const char *cl_tok_name(tok_kind_t kind);

#endif