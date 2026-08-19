#include "kstd.h"

extern void kprintf(const char *fmt, ...);

void tr_log(const char *s)
{
    kprintf("%s\n", s);
}