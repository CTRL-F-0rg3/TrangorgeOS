#include "alloc_debug.h"
#include "leak.h"
#include "stats.h"
#include "../heap/heap.h"
#include "../physical/pmm.h"
#include "../virtual/vmm.h"
#include "../heap/buddy.h"
#include "../heap/slab.h"
#include "../../cache/cache.h"

extern void kprintf(const char *fmt, ...);

#define DBG_MAGIC 0x444247414C4C4F43ULL
#define DBG_POISON_ALLOC 0xAA
#define DBG_POISON_FREE  0xFF

typedef struct dbg_header {
    uint64_t magic;
    uint64_t size;
    uint64_t caller;
    uint64_t reserved;
} dbg_header_t;

static uint64_t dbg_tail_canary(size_t size)
{
    return ~((uint64_t)size) ^ DBG_MAGIC;
}

static uint64_t *dbg_tail_ptr(void *user, size_t size)
{
    return (uint64_t *)((uint8_t *)user + size);
}

void *dbg_alloc(size_t size)
{
    if (size == 0) {
        return NULL;
    }

    uint64_t caller = (uint64_t)(uintptr_t)__builtin_return_address(0);

    uint8_t *real =
        (uint8_t *)heap_alloc(sizeof(dbg_header_t) + size + sizeof(uint64_t));

    if (real == NULL) {
        return NULL;
    }

    dbg_header_t *h = (dbg_header_t *)real;

    h->magic = DBG_MAGIC;
    h->size = size;
    h->caller = caller;
    h->reserved = 0;

    uint8_t *user = real + sizeof(dbg_header_t);

    for (size_t i = 0; i < size; i++) {
        user[i] = DBG_POISON_ALLOC;
    }

    *dbg_tail_ptr(user, size) = dbg_tail_canary(size);

    leak_track(user, size, caller);
    alloc_stats_note_alloc(size);

    return user;
}

void dbg_free(void *ptr)
{
    if (ptr == NULL) {
        return;
    }

    dbg_header_t *h =
        (dbg_header_t *)((uint8_t *)ptr - sizeof(dbg_header_t));

    if (h->magic != DBG_MAGIC) {
        kprintf("dbg_free: bad magic at %p\n", ptr);
        return;
    }

    size_t size = h->size;

    if (*dbg_tail_ptr(ptr, size) != dbg_tail_canary(size)) {
        kprintf("dbg_free: buffer overflow at %p caller 0x%llx\n",
                ptr,
                (unsigned long long)h->caller);
    }

    size_t leaked_size = 0;
    uint64_t alloc_caller = 0;

    if (!leak_untrack(ptr, &leaked_size, &alloc_caller)) {
        kprintf("dbg_free: double free or unknown ptr %p\n", ptr);
        return;
    }

    alloc_stats_note_free(size);

    h->magic = 0;

    uint8_t *real = (uint8_t *)h;
    size_t total = sizeof(dbg_header_t) + size + sizeof(uint64_t);

    for (size_t i = 0; i < total; i++) {
        real[i] = DBG_POISON_FREE;
    }

    heap_free(real);
}

bool dbg_verify(void *ptr)
{
    if (ptr == NULL) {
        return false;
    }

    dbg_header_t *h =
        (dbg_header_t *)((uint8_t *)ptr - sizeof(dbg_header_t));

    if (h->magic != DBG_MAGIC) {
        return false;
    }

    return *dbg_tail_ptr(ptr, h->size) == dbg_tail_canary(h->size);
}

void mm_debug_dump(void)
{
    alloc_stats_dump();
    leak_dump();
    pmm_dump();
    vmm_dump();
    buddy_dump();
    slab_dump();
    cache_dump();
}