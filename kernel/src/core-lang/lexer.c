#include "lexer.h"

typedef struct {
    const char *kw;
    tok_kind_t kind;
    uint16_t bits;
} kw_t;

static const kw_t KEYWORDS[] = {
    { "if",       TK_IF,       0 },
    { "else",     TK_ELSE,     0 },
    { "when",     TK_WHEN,     0 },
    { "case",     TK_CASE,     0 },
    { "fn",       TK_FN,       0 },
    { "let",      TK_LET,      0 },
    { "static",   TK_STATIC,   0 },
    { "data",     TK_DATA,     0 },
    { "dataimpl", TK_DATAIMPL, 0 },
    { "enum",     TK_ENUM,     0 },
    { "extern",   TK_EXTERN,   0 },
    { "call",     TK_CALL,     0 },
    { "set_free", TK_SET_FREE, 0 },

    { "u4",   TK_TYPE, 4 },
    { "u8",   TK_TYPE, 8 },
    { "u16",  TK_TYPE, 16 },
    { "u32",  TK_TYPE, 32 },
    { "u64",  TK_TYPE, 64 },
    { "u128", TK_TYPE, 128 },
    { "u256", TK_TYPE, 256 },
    { "u512", TK_TYPE, 512 },
    { "str",  TK_TYPE, 64 },
};

static char peek(const lexer_t *l)
{
    return l->pos < l->len ? l->src[l->pos] : '\0';
}

static char peek2(const lexer_t *l)
{
    return (l->pos + 1) < l->len ? l->src[l->pos + 1] : '\0';
}

static char take(lexer_t *l)
{
    char c = l->src[l->pos++];

    if (c == '\n') {
        l->line++;
        l->col = 1;
    } else {
        l->col++;
    }

    return c;
}

static void emit(lexer_t *l, token_t *t, tok_kind_t kind,
                 const char *start, uint32_t len,
                 uint32_t line, uint32_t col)
{
    t->kind = kind;
    t->start = start;
    t->len = len;
    t->num = 0;
    t->type_bits = 0;
    t->line = line;
    t->col = col;
}

static void skip_ws(lexer_t *l)
{
    for (;;) {
        char c = peek(l);

        if (c == ' ' || c == '\t' || c == '\r') {
            take(l);
        } else if (c == '\n') {
            take(l);
        } else if (c == '/' && peek2(l) == '/') {
            while (l->pos < l->len && peek(l) != '\n') {
                take(l);
            }
        } else if (c == '/' && peek2(l) == '*') {
            take(l);
            take(l);

            while (l->pos < l->len &&
                   !(peek(l) == '*' && peek2(l) == '/')) {
                take(l);
            }

            if (l->pos < l->len) {
                take(l);
                take(l);
            }
        } else {
            return;
        }
    }
}

static bool is_id_start(char c)
{
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

static bool is_id_char(char c)
{
    return is_id_start(c) || (c >= '0' && c <= '9');
}

static bool is_digit(char c)
{
    return c >= '0' && c <= '9';
}

static int hex_val(char c)
{
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

static void lex_number(lexer_t *l, token_t *t,
                       uint32_t line, uint32_t col)
{
    const char *start = l->src + l->pos;
    uint64_t val = 0;

    if (peek(l) == '0' && (peek2(l) == 'x' || peek2(l) == 'X')) {
        take(l);
        take(l);

        while (hex_val(peek(l)) >= 0) {
            val = (val << 4) | (uint64_t)hex_val(take(l));
        }
    } else if (peek(l) == '0' && (peek2(l) == 'b' || peek2(l) == 'B')) {
        take(l);
        take(l);

        while (peek(l) == '0' || peek(l) == '1') {
            val = (val << 1) | (uint64_t)(take(l) - '0');
        }
    } else {
        while (is_digit(peek(l))) {
            val = val * 10 + (uint64_t)(take(l) - '0');
        }
    }

    emit(l, t, TK_NUM, start,
         (uint32_t)(l->src + l->pos - start), line, col);
    t->num = val;
}

static void lex_ident(lexer_t *l, token_t *t,
                      uint32_t line, uint32_t col)
{
    const char *start = l->src + l->pos;

    while (is_id_char(peek(l))) {
        take(l);
    }

    uint32_t len = (uint32_t)(l->src + l->pos - start);

    for (size_t i = 0; i < sizeof(KEYWORDS) / sizeof(KEYWORDS[0]); i++) {
        const kw_t *k = &KEYWORDS[i];
        size_t kwlen = 0;

        while (k->kw[kwlen]) kwlen++;

        if (kwlen == len) {
            bool same = true;

            for (size_t j = 0; j < len; j++) {
                if (start[j] != k->kw[j]) {
                    same = false;
                    break;
                }
            }

            if (same) {
                emit(l, t, k->kind, start, len, line, col);
                t->type_bits = k->bits;
                return;
            }
        }
    }

    if (len == 1 && start[0] == '_') {
        emit(l, t, TK_UNDERSCORE, start, len, line, col);
        return;
    }

    emit(l, t, TK_IDENT, start, len, line, col);
}

static void lex_string(lexer_t *l, token_t *t,
                       uint32_t line, uint32_t col)
{
    take(l); 

    const char *start = l->src + l->pos;

    while (l->pos < l->len && peek(l) != '"') {
        if (peek(l) == '\\') {
            take(l);
        }

        take(l);
    }

    uint32_t len = (uint32_t)(l->src + l->pos - start);

    if (peek(l) == '"') {
        take(l);
    } else {
        emit(l, t, TK_ERR, start, len, line, col);
        return;
    }

    emit(l, t, TK_STR, start, len, line, col);
}

static void lex_char(lexer_t *l, token_t *t,
                     uint32_t line, uint32_t col)
{
    take(l); 

    uint64_t v = 0;

    if (peek(l) == '\\') {
        take(l);

        char e = take(l);

        switch (e) {
        case 'n': v = '\n'; break;
        case 't': v = '\t'; break;
        case '0': v = 0; break;
        case '\\': v = '\\'; break;
        case '\'': v = '\''; break;
        default: v = (uint64_t)e; break;
        }
    } else {
        v = (uint64_t)(unsigned char)take(l);
    }

    if (peek(l) == '\'') {
        take(l);
    } else {
        emit(l, t, TK_ERR, l->src + l->pos, 1, line, col);
        return;
    }

    emit(l, t, TK_CHAR, l->src + l->pos - 1, 1, line, col);
    t->num = v;
}

bool cl_lex_next(lexer_t *l, token_t *t)
{
    skip_ws(l);

    uint32_t line = l->line;
    uint32_t col = l->col;

    if (l->pos >= l->len) {
        emit(l, t, TK_EOF, l->src + l->pos, 0, line, col);
        return true;
    }

    char c = peek(l);

    if (is_digit(c)) {
        lex_number(l, t, line, col);
        return true;
    }

    if (is_id_start(c)) {
        lex_ident(l, t, line, col);
        return true;
    }

    if (c == '"') {
        lex_string(l, t, line, col);
        return t->kind != TK_ERR;
    }

    if (c == '\'') {
        lex_char(l, t, line, col);
        return t->kind != TK_ERR;
    }

    const char *start = l->src + l->pos;

    if (c == '$') {
        take(l);

        if (peek(l) == '!') {
            take(l);
            emit(l, t, TK_DOLLAR_BANG, start, 2, line, col);
        } else if (peek(l) == '@') {
            take(l);
            emit(l, t, TK_DOLLAR_AT, start, 2, line, col);
        } else {
            emit(l, t, TK_DOLLAR, start, 1, line, col);
        }

        return true;
    }

    char d = peek2(l);

    if (c == '=' && d == '=') { take(l); take(l); emit(l, t, TK_EQ, start, 2, line, col); return true; }
    if (c == '=' && d == '>') { take(l); take(l); emit(l, t, TK_ARROW, start, 2, line, col); return true; }
    if (c == '=' && d == ':') { take(l); take(l); emit(l, t, TK_EQCOLON, start, 2, line, col); return true; }
    if (c == '!' && d == '=') { take(l); take(l); emit(l, t, TK_NEQ, start, 2, line, col); return true; }
    if (c == '<' && d == '=') { take(l); take(l); emit(l, t, TK_LEQ, start, 2, line, col); return true; }
    if (c == '>' && d == '=') { take(l); take(l); emit(l, t, TK_GEQ, start, 2, line, col); return true; }
    if (c == '<' && d == '<') { take(l); take(l); emit(l, t, TK_SHL, start, 2, line, col); return true; }
    if (c == '>' && d == '>') { take(l); take(l); emit(l, t, TK_SHR, start, 2, line, col); return true; }
    if (c == ':' && d == ':') { take(l); take(l); emit(l, t, TK_DCOLON, start, 2, line, col); return true; }

    /* jednoznaki */
    take(l);

    switch (c) {
    case ';': emit(l, t, TK_SEMI, start, 1, line, col); return true;
    case ',': emit(l, t, TK_COMMA, start, 1, line, col); return true;
    case '.': emit(l, t, TK_DOT, start, 1, line, col); return true;
    case ':': emit(l, t, TK_COLON, start, 1, line, col); return true;
    case '=': emit(l, t, TK_ASSIGN, start, 1, line, col); return true;
    case '<': emit(l, t, TK_LT, start, 1, line, col); return true;
    case '>': emit(l, t, TK_GT, start, 1, line, col); return true;
    case '+': emit(l, t, TK_PLUS, start, 1, line, col); return true;
    case '-': emit(l, t, TK_MINUS, start, 1, line, col); return true;
    case '*': emit(l, t, TK_STAR, start, 1, line, col); return true;
    case '/': emit(l, t, TK_SLASH, start, 1, line, col); return true;
    case '%': emit(l, t, TK_PERCENT, start, 1, line, col); return true;
    case '&': emit(l, t, TK_AMP, start, 1, line, col); return true;
    case '|': emit(l, t, TK_PIPE, start, 1, line, col); return true;
    case '^': emit(l, t, TK_CARET, start, 1, line, col); return true;
    case '~': emit(l, t, TK_TILDE, start, 1, line, col); return true;
    case '!': emit(l, t, TK_BANG, start, 1, line, col); return true;
    case '{': emit(l, t, TK_LBRACE, start, 1, line, col); return true;
    case '}': emit(l, t, TK_RBRACE, start, 1, line, col); return true;
    case '[': emit(l, t, TK_LBRACK, start, 1, line, col); return true;
    case ']': emit(l, t, TK_RBRACK, start, 1, line, col); return true;
    case '(': emit(l, t, TK_LPAREN, start, 1, line, col); return true;
    case ')': emit(l, t, TK_RPAREN, start, 1, line, col); return true;
    default:
        emit(l, t, TK_ERR, start, 1, line, col);
        return false;
    }
}

void cl_lexer_init(lexer_t *l, const char *src, size_t len)
{
    l->src = src;
    l->len = len;
    l->pos = 0;
    l->line = 1;
    l->col = 1;
}

int cl_lex_all(lexer_t *l, token_t *buf, size_t cap, size_t *count)
{
    size_t n = 0;

    for (;;) {
        if (n >= cap) {
            return -1;
        }

        bool ok = cl_lex_next(l, &buf[n]);

        if (!ok) {
            return -2;
        }

        tok_kind_t k = buf[n].kind;

        n++;

        if (k == TK_EOF) {
            break;
        }
    }

    *count = n;
    return 0;
}

const char *cl_tok_name(tok_kind_t kind)
{
    switch (kind) {
    case TK_EOF: return "EOF";
    case TK_ERR: return "ERR";
    case TK_IDENT: return "IDENT";
    case TK_TYPE: return "TYPE";
    case TK_NUM: return "NUM";
    case TK_STR: return "STR";
    case TK_CHAR: return "CHAR";
    case TK_IF: return "if";
    case TK_ELSE: return "else";
    case TK_WHEN: return "when";
    case TK_CASE: return "case";
    case TK_FN: return "fn";
    case TK_LET: return "let";
    case TK_STATIC: return "static";
    case TK_DATA: return "data";
    case TK_DATAIMPL: return "dataimpl";
    case TK_ENUM: return "enum";
    case TK_EXTERN: return "extern";
    case TK_CALL: return "call";
    case TK_SET_FREE: return "set_free";
    case TK_SEMI: return ";";
    case TK_COMMA: return ",";
    case TK_DOT: return ".";
    case TK_COLON: return ":";
    case TK_DCOLON: return "::";
    case TK_ASSIGN: return "=";
    case TK_EQCOLON: return "=:";
    case TK_ARROW: return "=>";
    case TK_EQ: return "==";
    case TK_NEQ: return "!=";
    case TK_LT: return "<";
    case TK_GT: return ">";
    case TK_LEQ: return "<=";
    case TK_GEQ: return ">=";
    case TK_PLUS: return "+";
    case TK_MINUS: return "-";
    case TK_STAR: return "*";
    case TK_SLASH: return "/";
    case TK_PERCENT: return "%";
    case TK_AMP: return "&";
    case TK_PIPE: return "|";
    case TK_CARET: return "^";
    case TK_TILDE: return "~";
    case TK_BANG: return "!";
    case TK_SHL: return "<<";
    case TK_SHR: return ">>";
    case TK_LBRACE: return "{";
    case TK_RBRACE: return "}";
    case TK_LBRACK: return "[";
    case TK_RBRACK: return "]";
    case TK_LPAREN: return "(";
    case TK_RPAREN: return ")";
    case TK_UNDERSCORE: return "_";
    case TK_DOLLAR: return "$";
    case TK_DOLLAR_BANG: return "$!";
    case TK_DOLLAR_AT: return "$@";
    default: return "?";
    }
}