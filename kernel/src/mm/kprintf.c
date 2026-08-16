/*
 * Minimalny UART (COM1) + kprintf — mostek debugowy do portu szeregowego.
 * Nadpisuje słabą definicję kprintf z memory.c (silna definicja wygrywa).
 */

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdarg.h>

#define COM1 0x3F8

static inline uint8_t inb(uint16_t port)
{
    uint8_t v;

    __asm__ volatile("inb %%dx, %0" : "=a"(v) : "d"(port));

    return v;
}

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %%dx" : : "a"(val), "d"(port));
}

static bool uart_ready = false;

static void serial_init(void)
{
    if (uart_ready) {
        return;
    }

    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);

    uart_ready = true;
}

static void serial_putc(char c)
{
    while ((inb(COM1 + 5) & 0x20) == 0) {
    }

    outb(COM1, (uint8_t)c);
}

static void serial_puts(const char *s)
{
    while (*s) {
        serial_putc(*s++);
    }
}

static void serial_put_uint(uint64_t v, unsigned base, bool upper)
{
    char buf[32];
    int i = 32;

    if (v == 0) {
        serial_putc('0');
        return;
    }

    while (v > 0) {
        unsigned d = (unsigned)(v % base);

        buf[--i] = (char)(d < 10 ? '0' + d : (upper ? 'A' : 'a') + d - 10);
        v /= base;
    }

    while (i < 32) {
        serial_putc(buf[i++]);
    }
}

void kprintf(const char *fmt, ...)
{
    serial_init();

    va_list ap;

    va_start(ap, fmt);

    for (const char *p = fmt; *p; p++) {
        if (*p != '%') {
            serial_putc(*p);
            continue;
        }

        p++;
        if (*p == '\0') {
            break;
        }

        bool ll = false;
        bool l = false;
        bool z = false;

        while (*p == 'l' || *p == 'z' || *p == 'h') {
            if (*p == 'l') {
                if (p[1] == 'l') {
                    ll = true;
                    p++;
                } else {
                    l = true;
                }
            } else if (*p == 'z') {
                z = true;
            }
            p++;
        }

        switch (*p) {
        case 's': {
            const char *s = va_arg(ap, const char *);

            serial_puts(s ? s : "(null)");
            break;
        }
        case 'c':
            serial_putc((char)va_arg(ap, int));
            break;
        case 'd':
        case 'i': {
            int64_t v = ll ? (int64_t)va_arg(ap, long long)
                           : (l ? (int64_t)va_arg(ap, long)
                                : (int64_t)va_arg(ap, int));

            if (v < 0) {
                serial_putc('-');
                serial_put_uint((uint64_t)(-v), 10, false);
            } else {
                serial_put_uint((uint64_t)v, 10, false);
            }
            break;
        }
        case 'u': {
            uint64_t v = ll ? (uint64_t)va_arg(ap, unsigned long long)
                            : (z ? (uint64_t)va_arg(ap, size_t)
                                 : (l ? (uint64_t)va_arg(ap, unsigned long)
                                      : (uint64_t)va_arg(ap, unsigned int)));

            serial_put_uint(v, 10, false);
            break;
        }
        case 'x':
        case 'X': {
            uint64_t v = ll ? (uint64_t)va_arg(ap, unsigned long long)
                            : (z ? (uint64_t)va_arg(ap, size_t)
                                 : (l ? (uint64_t)va_arg(ap, unsigned long)
                                      : (uint64_t)va_arg(ap, unsigned int)));

            serial_put_uint(v, 16, *p == 'X');
            break;
        }
        case 'p': {
            void *ptr = va_arg(ap, void *);

            serial_puts("0x");
            serial_put_uint((uint64_t)(uintptr_t)ptr, 16, false);
            break;
        }
        case '%':
            serial_putc('%');
            break;
        default:
            serial_putc('%');
            serial_putc(*p);
            break;
        }
    }

    va_end(ap);
}

/* Dla strony Rust — prosty zapis łańcucha na port szeregowy. */
void serial_write_str(const char *s)
{
    serial_init();
    serial_puts(s);
}
