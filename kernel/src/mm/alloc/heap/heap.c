#include "heap.h"
#include "buddy.h"

#define HEAP_USE_SLAB 0

#if HEAP_USE_SLAB
#include "slab.h"
#endif

#include "../physical/pmm.h"
#include "../virtual/mapping.h"
#include "../../arch/x86_64/paging.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"

#define HEAP_BUDDY_BASE 0xFFFFB00000000000ULL
#define HEAP_BUDDY_SIZE (256ULL * 1024 * 1024)

#define HEAP_SLAB_BASE 0xFFFFC00000000000ULL
#define HEAP_SLAB_SIZE (64ULL * 1024 * 1024)

static bool heap_initialized = false;
static bool heap_slab_on = false;

static bool heap_map_cb(uint64_t virt, size_t size)
{
    size_t pages = size / ARCH_PAGE_SIZE;

    for (size_t i = 0; i < pages; i++) {
        uint64_t phys = 0;

        if (!pmm_alloc_frame(&phys)) {
            for (size_t j = 0; j < i; j++) {
                uint64_t v = virt + j * ARCH_PAGE_SIZE;
                uint64_t p = mapping_translate(MAPPING_KERNEL, v);

                mapping_unmap_range(MAPPING_KERNEL, v, ARCH_PAGE_SIZE);

                if (p != UINT64_MAX) {
                    pmm_free_frame(p);
                }
            }

            return false;
        }

        if (!mapping_map_range(MAPPING_KERNEL,
                               virt + i * ARCH_PAGE_SIZE,
                               phys,
                               ARCH_PAGE_SIZE,
                               PTE_PRESENT | PTE_WRITABLE | PTE_NX)) {
            pmm_free_frame(phys);

            for (size_t j = 0; j < i; j++) {
                uint64_t v = virt + j * ARCH_PAGE_SIZE;
                uint64_t p = mapping_translate(MAPPING_KERNEL, v);

                mapping_unmap_range(MAPPING_KERNEL, v, ARCH_PAGE_SIZE);

                if (p != UINT64_MAX) {
                    pmm_free_frame(p);
                }
            }

            return false;
        }
    }

    return true;
}

static void heap_unmap_cb(uint64_t virt, size_t size)
{
    size_t pages = size / ARCH_PAGE_SIZE;

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = virt + i * ARCH_PAGE_SIZE;
        uint64_t p = mapping_translate(MAPPING_KERNEL, v);

        if (p != UINT64_MAX) {
            mapping_unmap_range(MAPPING_KERNEL, v, ARCH_PAGE_SIZE);
            pmm_free_frame(p);
        }
    }
}

static void heap_memset(void *dst, uint8_t value, size_t n)
{
    uint8_t *p = (uint8_t *)dst;

    for (size_t i = 0; i < n; i++) {
        p[i] = value;
    }
}

static bool heap_in_slab(uint64_t va)
{
    return va >= HEAP_SLAB_BASE && va < HEAP_SLAB_BASE + HEAP_SLAB_SIZE;
}

static bool heap_in_buddy(uint64_t va)
{
    return va >= HEAP_BUDDY_BASE && va < HEAP_BUDDY_BASE + HEAP_BUDDY_SIZE;
}

bool heap_init(void)
{
    if (heap_initialized) {
        return true;
    }

    if (!pmm_ready()) {
        return false;
    }

    if (!buddy_init(HEAP_BUDDY_BASE,
                    HEAP_BUDDY_SIZE,
                    heap_map_cb,
                    heap_unmap_cb)) {
        return false;
    }

#if HEAP_USE_SLAB
    heap_slab_on = slab_init(HEAP_SLAB_BASE, HEAP_SLAB_SIZE);
#else
    heap_slab_on = false;
#endif

    heap_initialized = true;

    return true;
}

bool heap_ready(void)
{
    return heap_initialized;
}

void *heap_alloc(size_t size)
{
    if (!heap_initialized || size == 0) {
        return NULL;
    }

#if HEAP_USE_SLAB
    if (heap_slab_on && size <= SLAB_MAX_SIZE) {
        return slab_alloc(size);
    }
#endif

    return buddy_alloc(size);
}

/*
 * P1.1: `align` musi być potęgą dwójki — kontrakt jest teraz sprawdzany
 * na samym wejściu do warstwy heap, zanim trafi do gałęzi slab/buddy
 * (patrz uzasadnienie w buddy_alloc_aligned()).
 */
void *heap_alloc_aligned(size_t size, size_t align)
{
    if (!heap_initialized || size == 0) {
        return NULL;
    }

    if (!size_is_pow2(align)) {
        return NULL;
    }

#if HEAP_USE_SLAB
    if (heap_slab_on && size <= SLAB_MAX_SIZE && align <= 16) {
        return slab_alloc(size);
    }
#endif

    return buddy_alloc_aligned(size, align);
}

void *heap_zalloc(size_t size)
{
    void *ptr = heap_alloc(size);

    if (ptr == NULL) {
        return NULL;
    }

    heap_memset(ptr, 0, size);

    return ptr;
}

void heap_free(void *ptr)
{
    if (!heap_initialized || ptr == NULL) {
        return;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

#if HEAP_USE_SLAB
    if (heap_slab_on && heap_in_slab(va)) {
        slab_free(ptr);
        return;
    }
#endif

    if (heap_in_buddy(va)) {
        buddy_free(ptr);
    }
}

size_t heap_usable_size(void *ptr)
{
    if (!heap_initialized || ptr == NULL) {
        return 0;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

#if HEAP_USE_SLAB
    if (heap_slab_on && heap_in_slab(va)) {
        return slab_usable_size(ptr);
    }
#endif

    if (heap_in_buddy(va)) {
        return buddy_block_size(ptr);
    }

    return 0;
}

void heap_dump(void)
{
    if (!heap_initialized) {
        return;
    }

    buddy_dump();

#if HEAP_USE_SLAB
    if (heap_slab_on) {
        slab_dump();
    }
#endif
}