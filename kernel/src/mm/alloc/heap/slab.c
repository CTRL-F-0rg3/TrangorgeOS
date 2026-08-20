#include "slab.h"
#include "../physical/pmm.h"
#include "../physical/bitmap.h"
#include "../virtual/mapping.h"
#include "../../arch/x86_64/paging.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/smp_lock.h"

extern void kprintf(const char *fmt, ...);

#define SLAB_CACHE_COUNT 8

static const size_t slab_classes[SLAB_CACHE_COUNT] = {
    16, 32, 64, 128, 256, 512, 1024, 2048
};

/*
 * Liczba bitów w `used_bitmap` — musi pokryć najgorszy przypadek liczby
 * obiektów na slab, czyli najmniejszą klasę (16 B) w jednej stronie
 * (ARCH_PAGE_SIZE / 16 = 256 dla stron 4 KiB).
 */
#define SLAB_MAX_OBJECTS_PER_SLAB 256
#define SLAB_BITMAP_WORDS (SLAB_MAX_OBJECTS_PER_SLAB / 32)

typedef struct slab_desc {
    uint32_t cache_id;
    uint32_t free_count;
    uint32_t free_head;
    int32_t next_partial;
    uint32_t state;

    /*
     * P0.2: bitmapa zajętości per obiekt (1 = obiekt aktualnie
     * zaalokowany, 0 = wolny). Niezależna od intruzywnej listy wolnych
     * obiektów (`free_head`/`free_count`), więc pozwala wykryć
     * double-free: drugie `slab_free()` na ten sam wskaźnik znajdzie bit
     * już wyczyszczony i zostanie odrzucone, zamiast ponownie wpisać
     * adres do free-listy (co mogłoby podnieść `free_count` powyżej
     * `objects_per_slab` i uszkodzić listę wolnych obiektów).
     */
    uint32_t used_bitmap[SLAB_BITMAP_WORDS];
} slab_desc_t;

typedef struct slab_cache {
    size_t object_size;
    uint32_t objects_per_slab;
    int32_t partial_head;
    size_t total_slabs;
} slab_cache_t;

static uint64_t slab_base = 0;
static size_t slab_pages = 0;

static bitmap_t slab_page_bitmap;
static slab_desc_t *slab_descs = NULL;
static slab_cache_t slab_caches[SLAB_CACHE_COUNT];

static bool slab_initialized = false;

/*
 * P0.1: rzeczywista blokada SMP zamiast `pushfq; cli` + lokalnego
 * licznika, które nie chroniły przed drugim rdzeniem modyfikującym tę
 * samą free-listę/slab jednocześnie.
 */
static smp_ticket_lock_t slab_smp_lock = SMP_TICKET_LOCK_INIT;

static void slab_lock(void)
{
    smp_lock_acquire(&slab_smp_lock);
}

static void slab_unlock(void)
{
    smp_lock_release(&slab_smp_lock);
}

static inline bool slab_bit_test(const slab_desc_t *d, uint32_t idx)
{
    return ((d->used_bitmap[idx / 32] >> (idx % 32)) & 1u) != 0;
}

static inline void slab_bit_set(slab_desc_t *d, uint32_t idx)
{
    d->used_bitmap[idx / 32] |= (1u << (idx % 32));
}

static inline void slab_bit_clear(slab_desc_t *d, uint32_t idx)
{
    d->used_bitmap[idx / 32] &= ~(1u << (idx % 32));
}

static inline void slab_bitmap_clear_all(slab_desc_t *d)
{
    for (size_t i = 0; i < SLAB_BITMAP_WORDS; i++) {
        d->used_bitmap[i] = 0;
    }
}

static uint64_t slab_page_va(size_t page)
{
    return slab_base + (uint64_t)page * ARCH_PAGE_SIZE;
}

static void partial_remove(slab_cache_t *c, uint32_t page)
{
    int32_t *p = &c->partial_head;

    while (*p >= 0) {
        if ((uint32_t)*p == page) {
            *p = slab_descs[*p].next_partial;
            return;
        }

        p = &slab_descs[*p].next_partial;
    }
}

static bool slab_grow(uint32_t cache_id)
{
    slab_cache_t *c = &slab_caches[cache_id];

    size_t page = bitmap_alloc(&slab_page_bitmap);

    if (page == BITMAP_INVALID) {
        return false;
    }

    uint64_t va = slab_page_va(page);

    uint64_t phys = 0;

    if (!pmm_alloc_frame(&phys)) {
        bitmap_free(&slab_page_bitmap, page);
        return false;
    }

    if (!mapping_map_range(MAPPING_KERNEL,
                           va,
                           phys,
                           ARCH_PAGE_SIZE,
                           PTE_PRESENT | PTE_WRITABLE | PTE_NX)) {
        pmm_free_frame(phys);
        bitmap_free(&slab_page_bitmap, page);
        return false;
    }

    slab_desc_t *d = &slab_descs[page];

    d->cache_id = cache_id;
    d->state = 1;
    d->free_count = c->objects_per_slab;
    d->free_head = 0;

    /* Nowa (lub odzyskana z pustej) strona: wszystkie obiekty wolne. */
    slab_bitmap_clear_all(d);

    uint32_t *nexts = (uint32_t *)(uintptr_t)va;
    size_t stride = c->object_size / sizeof(uint32_t);

    for (uint32_t i = 0; i < c->objects_per_slab; i++) {
        nexts[i * stride] =
            (i + 1 < c->objects_per_slab) ? (i + 1) : UINT32_MAX;
    }

    d->next_partial = c->partial_head;
    c->partial_head = (int32_t)page;
    c->total_slabs++;

    return true;
}

bool slab_init(uint64_t base, size_t size)
{
    if (slab_initialized) {
        return true;
    }

    if (!pmm_ready()) {
        return false;
    }

    if (size == 0 || (size & (size - 1)) != 0) {
        return false;
    }

    if (size < ARCH_PAGE_SIZE) {
        return false;
    }

    if (!arch_is_page_aligned(base)) {
        return false;
    }

    if ((base & (size - 1)) != 0) {
        return false;
    }

    slab_base = base;
    slab_pages = size / ARCH_PAGE_SIZE;

    size_t bitmap_bytes = bitmap_bytes_for_bits(slab_pages);
    size_t bitmap_aligned = (bitmap_bytes + 7) & ~(size_t)7;
    size_t total = bitmap_aligned + slab_pages * sizeof(slab_desc_t);

    uint64_t meta_phys = 0;

    if (!pmm_alloc_bytes(total, &meta_phys)) {
        return false;
    }

    uint8_t *meta = (uint8_t *)arch_phys_to_virt(meta_phys);

    bitmap_init_virt(&slab_page_bitmap, meta, slab_pages);
    bitmap_fill(&slab_page_bitmap, false);

    slab_descs = (slab_desc_t *)(meta + bitmap_aligned);

    for (size_t i = 0; i < slab_pages; i++) {
        slab_descs[i].cache_id = 0;
        slab_descs[i].free_count = 0;
        slab_descs[i].free_head = UINT32_MAX;
        slab_descs[i].next_partial = -1;
        slab_descs[i].state = 0;
        slab_bitmap_clear_all(&slab_descs[i]);
    }

    for (size_t i = 0; i < SLAB_CACHE_COUNT; i++) {
        slab_caches[i].object_size = slab_classes[i];
        slab_caches[i].objects_per_slab =
            (uint32_t)(ARCH_PAGE_SIZE / slab_classes[i]);
        slab_caches[i].partial_head = -1;
        slab_caches[i].total_slabs = 0;

        /* objects_per_slab musi zmieścić się w used_bitmap (patrz P0.2). */
        if (slab_caches[i].objects_per_slab > SLAB_MAX_OBJECTS_PER_SLAB) {
            return false;
        }
    }

    slab_initialized = true;

    return true;
}

bool slab_ready(void)
{
    return slab_initialized;
}

void *slab_alloc(size_t size)
{
    if (!slab_initialized || size == 0 || size > SLAB_MAX_SIZE) {
        return NULL;
    }

    uint32_t cache_id = 0;

    while (cache_id < SLAB_CACHE_COUNT &&
           slab_caches[cache_id].object_size < size) {
        cache_id++;
    }

    if (cache_id >= SLAB_CACHE_COUNT) {
        return NULL;
    }

    slab_lock();

    slab_cache_t *c = &slab_caches[cache_id];

    if (c->partial_head < 0) {
        if (!slab_grow(cache_id)) {
            slab_unlock();
            return NULL;
        }
    }

    uint32_t page = (uint32_t)c->partial_head;
    slab_desc_t *d = &slab_descs[page];

    uint32_t idx = d->free_head;

    uint64_t va = slab_page_va(page);
    uint8_t *obj =
        (uint8_t *)(uintptr_t)(va + (uint64_t)idx * c->object_size);

    d->free_head = *(uint32_t *)(void *)obj;
    d->free_count--;

    /* P0.2: oznacz obiekt jako zajęty niezależnie od intruzywnej free-listy. */
    slab_bit_set(d, idx);

    if (d->free_count == 0) {
        c->partial_head = d->next_partial;
        d->next_partial = -1;
    }

    slab_unlock();

    return obj;
}

void slab_free(void *ptr)
{
    if (!slab_initialized || ptr == NULL) {
        return;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

    if (va < slab_base ||
        va >= slab_base + (uint64_t)slab_pages * ARCH_PAGE_SIZE) {
        return;
    }

    size_t page = (size_t)((va - slab_base) / ARCH_PAGE_SIZE);

    slab_desc_t *d = &slab_descs[page];

    if (d->state != 1) {
        return;
    }

    slab_cache_t *c = &slab_caches[d->cache_id];

    uint64_t slab_va = slab_page_va(page);
    uint64_t offset = va - slab_va;

    if (offset % c->object_size != 0) {
        return;
    }

    uint32_t idx = (uint32_t)(offset / c->object_size);

    if (idx >= c->objects_per_slab) {
        return;
    }

    slab_lock();

    if (!slab_bit_test(d, idx)) {
        kprintf("slab: double-free lub invalid-free wykryte: "
                "ptr=%p page=%zu idx=%u\n",
                ptr, page, (unsigned int)idx);
        slab_unlock();
        return;
    }

    slab_bit_clear(d, idx);

    bool was_full = (d->free_count == 0);

    *(uint32_t *)(uintptr_t)va = d->free_head;
    d->free_head = idx;
    d->free_count++;

    if (was_full) {
        d->next_partial = c->partial_head;
        c->partial_head = (int32_t)page;
    }

    if (d->free_count == c->objects_per_slab && c->total_slabs > 1) {
        partial_remove(c, page);

        uint64_t phys = mapping_translate(MAPPING_KERNEL, slab_va);

        mapping_unmap_range(MAPPING_KERNEL, slab_va, ARCH_PAGE_SIZE);

        if (phys != UINT64_MAX) {
            pmm_free_frame(phys);
        }

        bitmap_free(&slab_page_bitmap, page);

        d->state = 0;
        c->total_slabs--;
    }

    slab_unlock();
}

size_t slab_usable_size(void *ptr)
{
    if (!slab_initialized || ptr == NULL) {
        return 0;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

    if (va < slab_base ||
        va >= slab_base + (uint64_t)slab_pages * ARCH_PAGE_SIZE) {
        return 0;
    }

    size_t page = (size_t)((va - slab_base) / ARCH_PAGE_SIZE);

    const slab_desc_t *d = &slab_descs[page];

    if (d->state != 1) {
        return 0;
    }

    return slab_caches[d->cache_id].object_size;
}

size_t slab_stat_used_bytes(void)
{
    if (!slab_initialized) {
        return 0;
    }

    size_t used = 0;

    for (size_t i = 0; i < slab_pages; i++) {
        const slab_desc_t *d = &slab_descs[i];

        if (d->state != 1) {
            continue;
        }

        const slab_cache_t *c = &slab_caches[d->cache_id];

        used += (size_t)(c->objects_per_slab - d->free_count) *
                c->object_size;
    }

    return used;
}

size_t slab_stat_free_bytes(void)
{
    if (!slab_initialized) {
        return 0;
    }

    size_t free_bytes = 0;

    for (size_t i = 0; i < slab_pages; i++) {
        const slab_desc_t *d = &slab_descs[i];

        if (d->state != 1) {
            continue;
        }

        const slab_cache_t *c = &slab_caches[d->cache_id];

        free_bytes += (size_t)d->free_count * c->object_size;
    }

    return free_bytes;
}

void slab_dump(void)
{
    if (!slab_initialized) {
        kprintf("SLAB: not initialized\n");
        return;
    }

    kprintf("SLAB:\n");

    for (size_t i = 0; i < SLAB_CACHE_COUNT; i++) {
        kprintf("  size %4u: slabs %u\n",
                (unsigned int)slab_caches[i].object_size,
                (unsigned int)slab_caches[i].total_slabs);
    }

    kprintf("  used: %llu B\n",
            (unsigned long long)slab_stat_used_bytes());
    kprintf("  free in slabs: %llu B\n",
            (unsigned long long)slab_stat_free_bytes());
}