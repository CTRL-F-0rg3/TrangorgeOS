#include "frame.h"
#include "bitmap.h"


#include "../../arch/x86_64/memory.h"

static bitmap_t frame_bitmap;

static bool frame_initialized = false;

static size_t total_frames = 0;
static size_t allocated_frames = 0;

size_t frame_to_pfn(frame_t frame)
{
    return (size_t)(frame / ARCH_PAGE_SIZE);
}

frame_t frame_from_pfn(size_t pfn)
{
    return (frame_t)pfn * ARCH_PAGE_SIZE;
}

uint64_t frame_phys(frame_t frame)
{
    return frame;
}

void *frame_virt(frame_t frame)
{
    return arch_phys_to_virt(frame);
}

bool frame_is_valid(frame_t frame)
{
    if (!frame_initialized) {
        return false;
    }

    if (!arch_is_page_aligned(frame)) {
        return false;
    }

    size_t pfn = frame_to_pfn(frame);

    return pfn < total_frames;
}

bool frame_init(uint64_t bitmap_phys, size_t bit_count)
{
    if (frame_initialized) {
        return false;
    }

    if (bit_count == 0) {
        return false;
    }

    if (!bitmap_init_phys(&frame_bitmap, bitmap_phys, bit_count)) {
        return false;
    }

    /*
     * Na starcie wszystko jest zajęte.
     * Potem ręcznie zwolnimy regiony USABLE.
     */
    bitmap_fill(&frame_bitmap, true);

    total_frames = bit_count;
    allocated_frames = bit_count;

    frame_initialized = true;

    return true;
}

bool frame_init_from_memory(void)
{
    if (frame_initialized) {
        return false;
    }

    const arch_mem_info_t *info = arch_memory_get();

    if (info == NULL) {
        return false;
    }

    if (info->max_address < ARCH_PAGE_SIZE) {
        return false;
    }

    /*
     * Bitmapa indeksuje wszystkie możliwe PFN-y aż do max_address,
     * łącznie z dziurami w przestrzeni fizycznej.
     */
    uint64_t max_pfn = info->max_address / ARCH_PAGE_SIZE;
    size_t bit_count = (size_t)(max_pfn + 1);

    size_t bitmap_bytes = bitmap_bytes_for_bits(bit_count);

    uint64_t bitmap_phys = 0;

    if (!arch_memory_boot_alloc(bitmap_bytes,
                                ARCH_PAGE_SIZE,
                                &bitmap_phys)) {
        return false;
    }

    if (!frame_init(bitmap_phys, bit_count)) {
        return false;
    }

    /*
     * Zwalniamy tylko regiony USABLE.
     *
     * Uwaga: bitmapa została zarezerwowana przez
     * arch_memory_boot_alloc(), więc nie powinna zostać zwolniona.
     */
    const arch_mem_region_t *regions = NULL;
    size_t region_count = arch_memory_regions(&regions);

    for (size_t i = 0; i < region_count; i++) {
        const arch_mem_region_t *r = &regions[i];

        if (r->type != ARCH_MEM_TYPE_USABLE) {
            continue;
        }

        if (!arch_is_page_aligned(r->base) ||
            !arch_is_page_aligned(r->len)) {
            continue;
        }

        size_t pfn_start = (size_t)(r->base / ARCH_PAGE_SIZE);
        size_t frame_count = (size_t)(r->len / ARCH_PAGE_SIZE);

        if (frame_count == 0) {
            continue;
        }

        bitmap_clear_range(&frame_bitmap, pfn_start, frame_count);

        if (allocated_frames >= frame_count) {
            allocated_frames -= frame_count;
        } else {
            allocated_frames = 0;
        }
    }

    return true;
}

bool frame_ready(void)
{
    return frame_initialized;
}

bool frame_alloc(frame_t *out)
{
    if (!frame_initialized || out == NULL) {
        return false;
    }

    size_t pfn = bitmap_alloc(&frame_bitmap);

    if (pfn == BITMAP_INVALID) {
        return false;
    }

    allocated_frames++;

    *out = frame_from_pfn(pfn);

    return true;
}

bool frame_alloc_zero(frame_t *out)
{
    frame_t frame = FRAME_INVALID;

    if (!frame_alloc(&frame)) {
        return false;
    }

    if (!frame_zero(frame)) {
        frame_free(frame);
        return false;
    }

    *out = frame;

    return true;
}

bool frame_alloc_contiguous(size_t count,
                            size_t align_frames,
                            frame_t *out)
{
    if (!frame_initialized || out == NULL || count == 0) {
        return false;
    }

    if (count > total_frames) {
        return false;
    }

    if (align_frames == 0) {
        align_frames = 1;
    }

    size_t pfn = bitmap_alloc_range(&frame_bitmap,
                                    count,
                                    align_frames);

    if (pfn == BITMAP_INVALID) {
        return false;
    }

    allocated_frames += count;

    *out = frame_from_pfn(pfn);

    return true;
}

bool frame_free(frame_t frame)
{
    if (!frame_is_valid(frame)) {
        return false;
    }

    size_t pfn = frame_to_pfn(frame);

    /*
     * Jeśli bit jest 0, ramka już jest wolna.
     */
    if (!bitmap_test(&frame_bitmap, pfn)) {
        return false;
    }

    bitmap_free(&frame_bitmap, pfn);

    if (allocated_frames > 0) {
        allocated_frames--;
    }

    return true;
}

bool frame_free_contiguous(frame_t start, size_t count)
{
    if (!frame_initialized || count == 0) {
        return false;
    }

    if (!arch_is_page_aligned(start)) {
        return false;
    }

    size_t pfn_start = frame_to_pfn(start);

    if (pfn_start >= total_frames) {
        return false;
    }

    if (count > total_frames - pfn_start) {
        return false;
    }

    /*
     * Jeśli cały zakres jest już wolny, traktujemy to jako double free.
     */
    if (bitmap_test_range_free(&frame_bitmap, pfn_start, count)) {
        return false;
    }

    bitmap_free_range(&frame_bitmap, pfn_start, count);

    if (allocated_frames >= count) {
        allocated_frames -= count;
    } else {
        allocated_frames = 0;
    }

    return true;
}

bool frame_zero(frame_t frame)
{
    if (!frame_is_valid(frame)) {
        return false;
    }

    uint64_t *p = (uint64_t *)frame_virt(frame);

    for (size_t i = 0; i < (ARCH_PAGE_SIZE / sizeof(uint64_t)); i++) {
        p[i] = 0;
    }

    __asm__ volatile("" ::: "memory");

    return true;
}

bool frame_alloc_below(size_t count,
                       size_t align_frames,
                       uint64_t max_phys,
                       frame_t *out)
{
    if (!frame_initialized || count == 0 || out == NULL) {
        return false;
    }

    if (align_frames == 0) {
        align_frames = 1;
    }

    size_t max_pfn = (size_t)(max_phys / ARCH_PAGE_SIZE);

    size_t limit = total_frames < max_pfn ? total_frames : max_pfn;

    if (count > limit) {
        return false;
    }

    size_t pfn = 0;

    while (pfn <= limit - count) {
        size_t mask = align_frames - 1;
        size_t aligned = (pfn + mask) & ~mask;

        if (aligned > limit - count) {
            break;
        }

        if (bitmap_test_range_free(&frame_bitmap, aligned, count)) {
            bitmap_set_range(&frame_bitmap, aligned, count);

            allocated_frames += count;

            *out = frame_from_pfn(aligned);

            return true;
        }

        pfn = aligned + 1;
    }

    return false;
}

size_t frame_total(void)
{
    if (!frame_initialized) {
        return 0;
    }

    return total_frames;
}

size_t frame_allocated(void)
{
    if (!frame_initialized) {
        return 0;
    }

    return allocated_frames;
}

size_t frame_free_count(void)
{
    if (!frame_initialized) {
        return 0;
    }

    if (allocated_frames >= total_frames) {
        return 0;
    }

    return total_frames - allocated_frames;
}