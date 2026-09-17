#include "editor.h"
#include "mouse.h"
#include "../core-lang/lexer.h"
#include "../core-lang/loader.h"
#include "../core-lang/bridge.h"
#include "../core-lang/vm.h"

extern int gfx_fb_info_raw(uint32_t *w, uint32_t *h, uint32_t *s,
                           uint64_t *base, int32_t *flip);
extern const uint8_t font8x8[96][8];
extern void console_set_enabled(int on);
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
extern uint32_t k_input_keycode(void);


#define TOP_BAR 24
#define BOT_BAR 24
#define GUTTER  64
#define CHAR_W  16
#define CHAR_H  16
#define SCALE   2

#define COL_BG     0xFF1E1E1E
#define COL_FG     0xFFD4D4D4
#define COL_KEY    0xFF569CD6
#define COL_TYPE   0xFF4EC9B0
#define COL_NUM    0xFFB5CEA8
#define COL_STR    0xFFCE9178
#define COL_COM    0xFF6A9955
#define COL_OP     0xFFE0E0E0
#define COL_ID     0xFF9CDCFE
#define COL_GUTTER 0xFF858585
#define COL_BAR    0xFF252526
#define COL_CARET  0xFFAEAFAD


static char lines[ED_MAX_LINES][ED_LINE_LEN];
static int nlines = 1;
static int caret_l = 0, caret_c = 0;
static int top_line = 0;

static uint32_t *fb;
static uint32_t fw, fh, fstride;

static int mx = 200, my = 200;
static int mbuttons = 0;

static int ed_flip_rows = 0;

static char msg[ED_LINE_LEN] = "TrangEdit | F5=run F8=log ESC=exit";
static char log_lines[8][ED_LINE_LEN];
static int show_log = 0;

static uint8_t ed_arena_buf[196608];
static arena_t ed_arena;
static cl_vm_t ed_vm;
static char srcbuf[ED_MAX_LINES * (ED_LINE_LEN + 1)];


static size_t ed_strlen(const char *s)
{
    size_t n = 0;
    while (s[n]) n++;
    return n;
}

static void ed_fill(uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t c)
{
    for (uint32_t yy = y; yy < y + h && yy < fh; yy++) {
        uint32_t row = ed_flip_rows ? (fh - 1 - yy) : yy;

        for (uint32_t xx = x; xx < x + w && xx < fw; xx++) {
            fb[row * fstride + xx] = c;
        }
    }
}

static void ed_char(uint32_t x, uint32_t y, char ch, uint32_t color)
{
    if (ch < 32 || ch > 127) ch = '?';

    const uint8_t *g = font8x8[ch - 32];

    for (int ry = 0; ry < 8; ry++) {
        uint8_t bits = g[ry];

        for (int rx = 0; rx < 8; rx++) {
            if (bits & (0x80 >> rx)) {
                uint32_t px = x + (uint32_t)rx * SCALE;
                uint32_t py = y + (uint32_t)ry * SCALE;

                ed_fill(px, py, SCALE, SCALE, color);
            }
        }
    }
}

static void ed_text(uint32_t x, uint32_t y, const char *s, uint32_t color)
{
    while (*s) {
        ed_char(x, y, *s, color);
        x += CHAR_W;
        s++;
    }
}

static void ed_num(char *buf, int v)
{
    int i = 0;

    if (v == 0) { buf[0] = '0'; buf[1] = 0; return; }

    char tmp[12];
    int n = 0;

    while (v > 0 && n < 11) { tmp[n++] = (char)('0' + v % 10); v /= 10; }

    while (n > 0) buf[i++] = tmp[--n];
    buf[i] = 0;
}


static void log_append(const char *s)
{
    for (int i = 0; i < 7; i++) {
        for (int k = 0; k < ED_LINE_LEN; k++) {
            log_lines[i][k] = log_lines[i + 1][k];
        }
    }

    for (int k = 0; k < ED_LINE_LEN - 1 && s[k]; k++) {
        log_lines[7][k] = s[k];
        log_lines[7][k + 1] = 0;
    }
}

static uint64_t ed_put(uint64_t v, uint64_t b, uint64_t c,
                       uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;

    char buf[24];
    int i = 0;
    char tmp[24];
    int n = 0;

    if (v == 0) { tmp[n++] = '0'; }

    while (v > 0 && n < 20) { tmp[n++] = (char)('0' + v % 10); v /= 10; }

    while (n > 0) buf[i++] = tmp[--n];
    buf[i] = 0;

    log_append(buf);

    return 0;
}

static uint64_t ed_putc(uint64_t v, uint64_t b, uint64_t c,
                        uint64_t d, uint64_t e, uint64_t f)
{
    (void)b; (void)c; (void)d; (void)e; (void)f;

    char buf[2] = { (char)v, 0 };
    log_append(buf);
    return 0;
}


static uint32_t tok_color(tok_kind_t k)
{
    switch (k) {
    case TK_IF: case TK_ELSE: case TK_WHEN: case TK_CASE:
    case TK_FN: case TK_LET: case TK_STATIC: case TK_DATA:
    case TK_DATAIMPL: case TK_ENUM: case TK_EXTERN:
    case TK_CALL: case TK_SET_FREE:
        return COL_KEY;
    case TK_TYPE:   return COL_TYPE;
    case TK_NUM:    return COL_NUM;
    case TK_STR:
    case TK_CHAR:   return COL_STR;
    case TK_IDENT:  return COL_ID;
    default:        return COL_OP;
    }
}

static void draw_hl(int row, uint32_t x, uint32_t y)
{
    char tmp[ED_LINE_LEN + 1];
    int i = 0;

    for (; i < ED_LINE_LEN && lines[row][i]; i++) {
        tmp[i] = lines[row][i];
    }

    tmp[i] = 0;

    int com_at = -1;

    for (int k = 0; tmp[k]; k++) {
        if (tmp[k] == '/' && tmp[k + 1] == '/') { com_at = k; break; }
    }

    lexer_t l;
    token_t toks[64];
    size_t n = 0;

    cl_lexer_init(&l, tmp, ed_strlen(tmp));

    if (com_at >= 0) {
        char cut[ED_LINE_LEN + 1];

        for (int k = 0; k < com_at; k++) cut[k] = tmp[k];
        cut[com_at] = 0;

        cl_lexer_init(&l, cut, (size_t)com_at);
    }

    int pos = 0;

    if (cl_lex_all(&l, toks, 64, &n) == 0) {
        for (size_t t = 0; t < n && toks[t].kind != TK_EOF; t++) {
            int ts = (int)(toks[t].start - l.src);

            while (pos < ts) {
                ed_char(x + (uint32_t)pos * CHAR_W, y, ' ', COL_FG);
                pos++;
            }

            uint32_t col = tok_color(toks[t].kind);

            for (uint32_t k = 0; k < toks[t].len; k++) {
                ed_char(x + (uint32_t)pos * CHAR_W, y,
                        toks[t].start[k], col);
                pos++;
            }
        }
    }

    while (tmp[pos] && (com_at < 0 || pos < com_at)) {
        ed_char(x + (uint32_t)pos * CHAR_W, y, tmp[pos], COL_FG);
        pos++;
    }

    if (com_at >= 0) {
        for (int k = com_at; tmp[k]; k++) {
            ed_char(x + (uint32_t)k * CHAR_W, y, tmp[k], COL_COM);
        }
    }
}


static const char *CURSOR_SPRITE[] = {
    "X.........",
    "XX........",
    "XoX.......",
    "XooX......",
    "XoooX.....",
    "XooooX....",
    "XoooooX...",
    "XooooooX..",
    "XoooooooX.",
    "XooooooooX",
    "XoooooXXXX",
    "XooXooX...",
    "XoX.XooX..",
    "XX..XooX..",
    "X....XooX.",
    "......XX..",
};

static void draw_mouse(void)
{
    for (int ry = 0; ry < 16; ry++) {
        for (int rx = 0; rx < 10; rx++) {
            char c = CURSOR_SPRITE[ry][rx];

            if (c == '.') continue;

            uint32_t col = (c == 'X') ? 0xFF000000 : 0xFFFFFFFF;

            int sy = my + ry;
            int sx = mx + rx;

            if (sx >= 0 && sx < (int)fw && sy >= 0 && sy < (int)fh) {
                int row = ed_flip_rows ? (int)fh - 1 - sy : sy;
                fb[row * fstride + sx] = col;
            }
        }
    }
}


static void render(void)
{
    ed_fill(0, 0, fw, fh, COL_BG);

    ed_fill(0, 0, fw, TOP_BAR, COL_BAR);
    ed_text(8, 4, "TrangEdit [core-lang]", COL_TYPE);
    ed_text(GUTTER + 160, 4, msg, COL_GUTTER);

    int rows = (int)((fh - TOP_BAR - BOT_BAR) / CHAR_H);
    int cols = (int)((fw - GUTTER) / CHAR_W);

    for (int r = 0; r < rows; r++) {
        int row = top_line + r;

        if (row >= nlines) break;

        uint32_t y = TOP_BAR + (uint32_t)r * CHAR_H;

        if (row == caret_l) {
            ed_fill(GUTTER, y, (uint32_t)cols * CHAR_W, CHAR_H, 0xFF2D2D2D);
        }

        char nb[12];
        ed_num(nb, row + 1);
        ed_text(8, y, nb, COL_GUTTER);

        draw_hl(row, GUTTER + 4, y);
    }

    {
        int r = caret_l - top_line;

        if (r >= 0 && r < rows) {
            uint32_t cx = GUTTER + 4 + (uint32_t)caret_c * CHAR_W;
            uint32_t cy = TOP_BAR + (uint32_t)r * CHAR_H;

            ed_fill(cx, cy, 2, CHAR_H, COL_CARET);
        }
    }

    if (show_log) {
        uint32_t ly = fh - BOT_BAR - 8 * CHAR_H;

        ed_fill(0, ly, fw, 8 * CHAR_H, 0xFF101010);

        for (int i = 0; i < 8; i++) {
            ed_text(8, ly + (uint32_t)i * CHAR_H, log_lines[i], COL_NUM);
        }
    }

    ed_fill(0, fh - BOT_BAR, fw, BOT_BAR, COL_BAR);

    char st[64];
    int i = 0;
    const char *p1 = "ln ";
    while (*p1) st[i++] = *p1++;
    char tmp[12];
    ed_num(tmp, caret_l + 1);
    for (int k = 0; tmp[k]; k++) st[i++] = tmp[k];
    const char *p2 = " col ";
    while (*p2) st[i++] = *p2++;
    ed_num(tmp, caret_c + 1);
    for (int k = 0; tmp[k]; k++) st[i++] = tmp[k];
    st[i] = 0;

    ed_text(8, fh - BOT_BAR + 4, st, COL_FG);
    ed_text(GUTTER + 200, fh - BOT_BAR + 4, msg, COL_GUTTER);

    draw_mouse();
}


static void ins_char(char ch)
{
    char *ln = lines[caret_l];
    int len = (int)ed_strlen(ln);

    if (len >= ED_LINE_LEN - 1 || caret_c > len) return;

    for (int i = len; i >= caret_c; i--) {
        ln[i + 1] = ln[i];
    }

    ln[caret_c] = ch;
    caret_c++;
}

static void do_enter(void)
{
    if (nlines >= ED_MAX_LINES) return;

    for (int i = nlines; i > caret_l + 1; i--) {
        for (int k = 0; k < ED_LINE_LEN; k++) {
            lines[i][k] = lines[i - 1][k];
        }
    }

    nlines++;

    char *cur = lines[caret_l];
    char *nxt = lines[caret_l + 1];

    int j = 0;

    for (int k = caret_c; cur[k] && k < ED_LINE_LEN; k++) {
        nxt[j++] = cur[k];
        cur[k] = 0;
    }

    nxt[j] = 0;

    caret_l++;
    caret_c = 0;
}

static void do_backspace(void)
{
    char *ln = lines[caret_l];

    if (caret_c > 0) {
        int len = (int)ed_strlen(ln);

        for (int i = caret_c - 1; i < len; i++) {
            ln[i] = ln[i + 1];
        }

        caret_c--;
    } else if (caret_l > 0) {
        int plen = (int)ed_strlen(lines[caret_l - 1]);
        int slen = (int)ed_strlen(ln);

        if (plen + slen < ED_LINE_LEN - 1) {
            for (int k = 0; k <= slen; k++) {
                lines[caret_l - 1][plen + k] = ln[k];
            }

            for (int i = caret_l; i < nlines - 1; i++) {
                for (int k = 0; k < ED_LINE_LEN; k++) {
                    lines[i][k] = lines[i + 1][k];
                }
            }

            nlines--;
            caret_l--;
            caret_c = plen;
        }
    }
}

static void do_delete(void)
{
    char *ln = lines[caret_l];
    int len = (int)ed_strlen(ln);

    if (caret_c < len) {
        for (int i = caret_c; i < len; i++) {
            ln[i] = ln[i + 1];
        }
    }
}

static void clamp_scroll(int rows)
{
    if (caret_l < top_line) top_line = caret_l;
    if (caret_l >= top_line + rows) top_line = caret_l - rows + 1;
    if (top_line < 0) top_line = 0;
    if (top_line > nlines - 1) top_line = nlines - 1;
}


static int flatten(void)
{
    int o = 0;

    for (int i = 0; i < nlines; i++) {
        for (int k = 0; lines[i][k] && o < (int)sizeof(srcbuf) - 2; k++) {
            srcbuf[o++] = lines[i][k];
        }

        srcbuf[o++] = '\n';
    }

    srcbuf[o] = 0;
    return o;
}

static void ed_compile_run(void)
{
    const char *m0 = "compiling...";
    for (int i = 0; m0[i]; i++) { msg[i] = m0[i]; msg[i + 1] = 0; }

    render();

    ed_arena.pos = 0;

    int n = flatten();

    uint32_t el = 0;
    const char *em = (void *)0;

    cl_prog_t *P = cl_compile_source(srcbuf, (size_t)n, &ed_arena, &el, &em);

    if (P == (void *)0) {
        const char *pre = "ERR ln ";
        int i = 0;
        while (pre[i]) { msg[i] = pre[i]; i++; }
        char tmp[12];
        ed_num(tmp, (int)el);
        for (int k = 0; tmp[k]; k++) msg[i++] = tmp[k];
        msg[i++] = ' ';
        if (em) for (int k = 0; em[k] && i < ED_LINE_LEN - 1; k++) msg[i++] = em[k];
        msg[i] = 0;

        if (em) log_append(em);
        return;
    }

    cl_vm_init(&ed_vm, P);
    cl_bridge_init(&ed_vm, 0);

    cl_vm_register_extern(&ed_vm, "put", ed_put);
    cl_vm_register_extern(&ed_vm, "putc", ed_putc);

    log_append("== run ==");

    cl_vm_err_t e = cl_vm_run(&ed_vm);

    const char *ok = (e == CL_OK) ? "run: ok" : "run: vm error";
    int i = 0;
    while (ok[i]) { msg[i] = ok[i]; i++; }
    msg[i] = 0;

    show_log = 1;
}


int editor_run(const char *path)
{
    uint32_t w = 0, h = 0, s = 0;
    uint64_t base = 0;
    int32_t fliprows = 0;

    if (gfx_fb_info_raw(&w, &h, &s, &base, &fliprows) != 0 || base == 0) {
        return -1;
    }

    fb = (uint32_t *)(uintptr_t)base;
    fw = w; fh = h; fstride = s;
    ed_flip_rows = fliprows;

    nlines = 1;
    lines[0][0] = 0;
    caret_l = caret_c = 0;
    top_line = 0;

    if (path != (void *)0) {
        static char rbuf[ED_MAX_LINES * ED_LINE_LEN];

        int32_t n = k_fs_read(path, rbuf, sizeof(rbuf) - 1);

        if (n > 0) {
            rbuf[n] = 0;

            nlines = 0;
            int col = 0;

            for (int i = 0; i < n && nlines < ED_MAX_LINES; i++) {
                char c = rbuf[i];

                if (c == '\n') {
                    lines[nlines][col] = 0;
                    nlines++;
                    col = 0;
                } else if (col < ED_LINE_LEN - 1) {
                    lines[nlines][col++] = c;
                }
            }

            if (col > 0 || nlines == 0) {
                lines[nlines][col] = 0;
                nlines++;
            }
        }
    }

    console_set_enabled(0);
    mouse_init();

    int running = 1;

    while (running) {
        int dirty = 0;
        int rows = (int)((fh - TOP_BAR - BOT_BAR) / CHAR_H);

        int dx = 0, dy = 0, dz = 0, btn = 0;

        if (mouse_poll(&dx, &dy, &dz, &btn)) {
            mx += dx;
            my += dy;

            if (mx < 0) mx = 0;
            if (my < 0) my = 0;
            if (mx >= (int)fw) mx = (int)fw - 1;
            if (my >= (int)fh) my = (int)fh - 1;

            if (dz != 0) {
                top_line += dz;
                clamp_scroll(rows);
                dirty = 1;
            }

            if (btn & 1 && !(mbuttons & 1)) {
                if (my >= (int)TOP_BAR && my < (int)(fh - BOT_BAR)) {
                    int row = top_line + (my - (int)TOP_BAR) / CHAR_H;
                    int col = (mx - (int)GUTTER - 4) / CHAR_W;

                    if (row >= 0 && row < nlines) caret_l = row;

                    int len = (int)ed_strlen(lines[caret_l]);

                    if (col < 0) col = 0;
                    if (col > len) col = len;

                    caret_c = col;
                    clamp_scroll(rows);
                    dirty = 1;
                }
            }

            mbuttons = btn;
            dirty = 1;
        }

        uint32_t k = k_input_keycode();

        if (k != 0) {
            switch (k) {
            case EDK_ESC:       running = 0; break;
            case EDK_ENTER:     do_enter(); dirty = 1; break;
            case EDK_BACKSPACE: do_backspace(); dirty = 1; break;
            case EDK_DELETE:    do_delete(); dirty = 1; break;
            case EDK_TAB:       ins_char(' '); ins_char(' ');
                                ins_char(' '); ins_char(' '); dirty = 1; break;
            case EDK_LEFT:  if (caret_c > 0) caret_c--; dirty = 1; break;
            case EDK_RIGHT: caret_c++; dirty = 1; break;
            case EDK_UP:    if (caret_l > 0) caret_l--; dirty = 1; break;
            case EDK_DOWN:  if (caret_l < nlines - 1) caret_l++; dirty = 1; break;
            case EDK_HOME:  caret_c = 0; dirty = 1; break;
            case EDK_END:   caret_c = (int)ed_strlen(lines[caret_l]); dirty = 1; break;
            case EDK_F5:    ed_compile_run(); dirty = 1; break;
            case EDK_F8:    show_log = !show_log; dirty = 1; break;
            default:
                if (k >= 32 && k < 0x100) {
                    ins_char((char)k);
                    dirty = 1;
                }
                break;
            }

            clamp_scroll(rows);
        }

        if (dirty) {
            render();
        }

        __asm__ volatile("pause");
    }

    console_set_enabled(1);

    return 0;
}