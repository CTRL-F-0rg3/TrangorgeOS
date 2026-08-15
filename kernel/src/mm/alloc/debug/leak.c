#include "leak.h"

extern void kprintf(const char *fmt, ...);

#define LEAK_MAX 2048

typedef struct leak_entry {
    void *ptr;
    size_t size;
    uint64_t caller;
    bool used;
} leak_entry_t;

static leak_entry_t leak_table[LEAK_MAX];

bool leak_track(void *ptr, size_t size, uint64_t caller)
{
    for (size_t i = 0; i < LEAK_MAX; i++) {
        if (!leak_table[i].used) {
            leak_table[i].ptr = ptr;
            leak_table[i].size = size;
            leak_table[i].caller = caller;
            leak_table[i].used = true;
            return true;
        }
    }

    return false;
}

bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller)
{
    for (size_t i = 0; i < LEAK_MAX; i++) {
        if (leak_table[i].used && leak_table[i].ptr == ptr) {
            if (out_size != NULL) {
                *out_size = leak_table[i].size;
            }

            if (out_caller != NULL) {
                *out_caller = leak_table[i].caller;
            }

            leak_table[i].used = false;
            return true;
        }
    }

    return false;
}

bool leak_contains(void *ptr)
{
    for (size_t i = 0; i < LEAK_MAX; i++) {
        if (leak_table[i].used && leak_table[i].ptr == ptr) {
            return true;
        }
    }

    return false;
}

size_t leak_count(void)
{
    size_t n = 0;

    for (size_t i = 0; i < LEAK_MAX; i++) {
        if (leak_table[i].used) {
            n++;
        }
    }

    return n;
}

void leak_dump(void)
{
    kprintf("LEAKS: %llu live\n", (unsigned long long)leak_count());

    for (size_t i = 0; i < LEAK_MAX; i++) {
        if (!leak_table[i].used) {
            continue;
        }

        kprintf("  leak: %p size %llu caller 0x%llx\n",
                leak_table[i].ptr,
                (unsigned long long)leak_table[i].size,
                (unsigned long long)leak_table[i].caller);
    }
}