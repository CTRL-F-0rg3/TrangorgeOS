#include "alloc.h"
#include "../heap/heap.h"
#include "../debug/alloc_debug.h"
#include "../virtual/mapping.h"
#include "../../arch/x86_64/memory.h"

static void k_memset(void *dst, uint8_t value, size_t n)
{
    uint8_t *p = (uint8_t *)dst;

    for (size_t i = 0; i < n; i++) {
        p[i] = value;
    }
}

static void k_memcpy(void *dst, const void *src, size_t n)
{
    uint8_t *d = (uint8_t *)dst;
    const uint8_t *s = (const uint8_t *)src;

    for (size_t i = 0; i < n; i++) {
        d[i] = s[i];
    }
}

static size_t k_strlen(const char *s)
{
    size_t n = 0;

    while (s[n] != '\0') {
        n++;
    }

    return n;
}

static size_t k_usable_size(void *ptr)
{
#ifdef ALLOC_DEBUG
    size_t s = dbg_usable_size(ptr);

    if (s != 0) {
        return s;
    }
#endif

    return heap_usable_size(ptr);
}

void *kmalloc(size_t size)
{
    if (size == 0) {
        return NULL;
    }

#ifdef ALLOC_DEBUG
    return dbg_alloc(size);
#else
    return heap_alloc(size);
#endif
}

void kfree(void *ptr)
{
    if (ptr == NULL) {
        return;
    }

#ifdef ALLOC_DEBUG
    dbg_free(ptr);
#else
    heap_free(ptr);
#endif
}

void *kzalloc(size_t size)
{
    void *ptr = kmalloc(size);

    if (ptr == NULL) {
        return NULL;
    }

    k_memset(ptr, 0, size);

    return ptr;
}

void *kcalloc(size_t count, size_t size)
{
    if (count == 0 || size == 0) {
        return NULL;
    }

    if (count > SIZE_MAX / size) {
        return NULL;
    }

    return kzalloc(count * size);
}

void *kmalloc_aligned(size_t size, size_t align)
{
    if (size == 0) {
        return NULL;
    }

    return heap_alloc_aligned(size, align);
}

void *krealloc(void *ptr, size_t new_size)
{
    if (ptr == NULL) {
        return kmalloc(new_size);
    }

    if (new_size == 0) {
        kfree(ptr);
        return NULL;
    }

    size_t old_size = k_usable_size(ptr);

    if (old_size == 0) {
        return NULL;
    }

    if (old_size >= new_size) {
        return ptr;
    }

    void *new_ptr = kmalloc(new_size);

    if (new_ptr == NULL) {
        return NULL;
    }

    k_memcpy(new_ptr, ptr, old_size);

    kfree(ptr);

    return new_ptr;
}

void *kalloc_pages(size_t pages)
{
    if (pages == 0) {
        return NULL;
    }

    if (pages > SIZE_MAX / ARCH_PAGE_SIZE) {
        return NULL;
    }

    return heap_alloc(pages * ARCH_PAGE_SIZE);
}

void kfree_pages(void *ptr, size_t pages)
{
    (void)pages;

    heap_free(ptr);
}

char *kstrdup(const char *s)
{
    if (s == NULL) {
        return NULL;
    }

    size_t n = k_strlen(s) + 1;

    char *ptr = (char *)kmalloc(n);

    if (ptr == NULL) {
        return NULL;
    }

    k_memcpy(ptr, s, n);

    return ptr;
}

uint64_t kvirt_to_phys(void *ptr)
{
    if (ptr == NULL) {
        return UINT64_MAX;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

    uint64_t phys = mapping_translate(MAPPING_KERNEL, va);

    if (phys != UINT64_MAX) {
        return phys;
    }

    return arch_virt_to_phys(ptr);
}

void kalloc_dump(void)
{
#ifdef ALLOC_DEBUG
    mm_debug_dump();
#else
    heap_dump();
#endif
}