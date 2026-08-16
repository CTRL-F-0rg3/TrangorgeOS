#include "bitmap.h"

/*
 * Note: if your include path is different, fix this path or use -I flags.
 */
#include "../../arch/x86_64/memory.h"

#define BITMAP_SIZE_MAX ((size_t)-1)

static inline size_t size_min(size_t a, size_t b)
{
    return (a < b) ? a : b;
}

static inline size_t align_up_size(size_t value, size_t align)
{
    if (align == 0) {
        return value;
    }

    size_t mask = align - 1;

    if (value > BITMAP_SIZE_MAX - mask) {
        return BITMAP_SIZE_MAX;
    }

    return (value + mask) & ~mask;
}

static inline uint64_t mask_from_bit(size_t bit)
{
    if (bit == 0) {
        return UINT64_MAX;
    }

    return UINT64_MAX << bit;
}

static inline uint64_t mask_to_bit(size_t bit)
{
    if (bit == 63) {
        return UINT64_MAX;
    }

    return (1ULL << (bit + 1)) - 1;
}

static inline uint64_t lower_bits_mask(size_t bit)
{
    if (bit == 0) {
        return 0;
    }

    return (1ULL << bit) - 1;
}

size_t bitmap_words_for_bits(size_t bit_count)
{
    if (bit_count == 0) {
        return 0;
    }

    return (bit_count + 63) / 64;
}

size_t bitmap_bytes_for_bits(size_t bit_count)
{
    return bitmap_words_for_bits(bit_count) * sizeof(uint64_t);
}

static void bitmap_mark_tail_used(bitmap_t *bm)
{
    if (bm->word_count == 0) {
        return;
    }

    size_t rem = bm->bit_count & 63;

    if (rem == 0) {
        return;
    }

    /*
     * Set the bits above bit_count to 1 so the allocator never hands them
     * out.
     */
    bm->bits[bm->word_count - 1] |= ~((1ULL << rem) - 1);
}

void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count)
{
    if (bm == NULL || storage == NULL) {
        return;
    }

    bm->bits = (uint64_t *)storage;
    bm->bit_count = bit_count;
    bm->word_count = bitmap_words_for_bits(bit_count);
    bm->alloc_hint = 0;

    for (size_t i = 0; i < bm->word_count; i++) {
        bm->bits[i] = 0;
    }

    bitmap_mark_tail_used(bm);
}

bool bitmap_init_phys(bitmap_t *bm, uint64_t storage_phys, size_t bit_count)
{
    if (bm == NULL) {
        return false;
    }

    void *virt = arch_phys_to_virt(storage_phys);

    if (virt == NULL) {
        return false;
    }

    bitmap_init_virt(bm, virt, bit_count);

    return true;
}

void bitmap_fill(bitmap_t *bm, bool value)
{
    if (bm == NULL || bm->bits == NULL) {
        return;
    }

    uint64_t v = value ? UINT64_MAX : 0;

    for (size_t i = 0; i < bm->word_count; i++) {
        bm->bits[i] = v;
    }

    if (!value) {
        bitmap_mark_tail_used(bm);
    }
}


void bitmap_set(bitmap_t *bm, size_t bit)
{
    if (bm == NULL || bm->bits == NULL || bit >= bm->bit_count) {
        return;
    }

    size_t word = bit / 64;
    size_t offset = bit % 64;

    bm->bits[word] |= (1ULL << offset);
}

void bitmap_clear(bitmap_t *bm, size_t bit)
{
    if (bm == NULL || bm->bits == NULL || bit >= bm->bit_count) {
        return;
    }

    size_t word = bit / 64;
    size_t offset = bit % 64;

    bm->bits[word] &= ~(1ULL << offset);
}

bool bitmap_test(const bitmap_t *bm, size_t bit)
{
    if (bm == NULL || bm->bits == NULL || bit >= bm->bit_count) {
        return false;
    }

    size_t word = bit / 64;
    size_t offset = bit % 64;

    return (bm->bits[word] & (1ULL << offset)) != 0;
}

void bitmap_set_range(bitmap_t *bm, size_t start, size_t count)
{
    if (bm == NULL || bm->bits == NULL || count == 0) {
        return;
    }

    if (start >= bm->bit_count) {
        return;
    }

    if (count > bm->bit_count - start) {
        count = bm->bit_count - start;
    }

    if (count == 0) {
        return;
    }

    size_t end = start + count - 1;

    size_t first_word = start / 64;
    size_t last_word = end / 64;

    size_t first_bit = start % 64;
    size_t last_bit = end % 64;

    uint64_t first_mask = mask_from_bit(first_bit);
    uint64_t last_mask = mask_to_bit(last_bit);

    if (first_word == last_word) {
        bm->bits[first_word] |= (first_mask & last_mask);
        return;
    }

    bm->bits[first_word] |= first_mask;

    for (size_t i = first_word + 1; i < last_word; i++) {
        bm->bits[i] = UINT64_MAX;
    }

    bm->bits[last_word] |= last_mask;
}

void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count)
{
    if (bm == NULL || bm->bits == NULL || count == 0) {
        return;
    }

    if (start >= bm->bit_count) {
        return;
    }

    if (count > bm->bit_count - start) {
        count = bm->bit_count - start;
    }

    if (count == 0) {
        return;
    }

    size_t end = start + count - 1;

    size_t first_word = start / 64;
    size_t last_word = end / 64;

    size_t first_bit = start % 64;
    size_t last_bit = end % 64;

    uint64_t first_mask = mask_from_bit(first_bit);
    uint64_t last_mask = mask_to_bit(last_bit);

    if (first_word == last_word) {
        bm->bits[first_word] &= ~(first_mask & last_mask);
        return;
    }

    bm->bits[first_word] &= ~first_mask;

    for (size_t i = first_word + 1; i < last_word; i++) {
        bm->bits[i] = 0;
    }

    bm->bits[last_word] &= ~last_mask;
}

bool bitmap_test_range_free(const bitmap_t *bm, size_t start, size_t count)
{
    if (bm == NULL || bm->bits == NULL) {
        return false;
    }

    if (count == 0) {
        return true;
    }

    if (start >= bm->bit_count) {
        return false;
    }

    if (count > bm->bit_count - start) {
        return false;
    }

    size_t end = start + count - 1;

    size_t first_word = start / 64;
    size_t last_word = end / 64;

    size_t first_bit = start % 64;
    size_t last_bit = end % 64;

    uint64_t first_mask = mask_from_bit(first_bit);
    uint64_t last_mask = mask_to_bit(last_bit);

    if (first_word == last_word) {
        uint64_t mask = first_mask & last_mask;
        return (bm->bits[first_word] & mask) == 0;
    }

    if ((bm->bits[first_word] & first_mask) != 0) {
        return false;
    }

    for (size_t i = first_word + 1; i < last_word; i++) {
        if (bm->bits[i] != 0) {
            return false;
        }
    }

    if ((bm->bits[last_word] & last_mask) != 0) {
        return false;
    }

    return true;
}

static size_t find_next_zero_bit(const bitmap_t *bm, size_t start)
{
    if (bm == NULL || bm->bits == NULL || bm->bit_count == 0) {
        return BITMAP_INVALID;
    }

    if (start >= bm->bit_count) {
        return BITMAP_INVALID;
    }

    size_t word = start / 64;
    size_t bit = start % 64;

    uint64_t w = bm->bits[word] | lower_bits_mask(bit);

    if (~w != 0) {
        size_t found_bit = (size_t)__builtin_ctzll(~w);
        size_t idx = word * 64 + found_bit;

        if (idx < bm->bit_count) {
            return idx;
        }

        return BITMAP_INVALID;
    }

    for (size_t i = word + 1; i < bm->word_count; i++) {
        w = bm->bits[i];

        if (~w != 0) {
            size_t found_bit = (size_t)__builtin_ctzll(~w);
            size_t idx = i * 64 + found_bit;

            if (idx < bm->bit_count) {
                return idx;
            }

            return BITMAP_INVALID;
        }
    }

    return BITMAP_INVALID;
}

static size_t find_next_used_bit(const bitmap_t *bm, size_t start, size_t limit)
{
    if (bm == NULL || bm->bits == NULL || bm->bit_count == 0) {
        return BITMAP_INVALID;
    }

    if (limit > bm->bit_count) {
        limit = bm->bit_count;
    }

    if (start >= limit) {
        return BITMAP_INVALID;
    }

    size_t start_word = start / 64;
    size_t start_bit = start % 64;

    size_t end_word = (limit - 1) / 64;
    size_t end_bit = (limit - 1) % 64;

    for (size_t i = start_word; i <= end_word; i++) {
        uint64_t w = bm->bits[i];

        if (i == start_word && start_bit != 0) {
            w &= ~lower_bits_mask(start_bit);
        }

        if (i == end_word) {
            w &= mask_to_bit(end_bit);
        }

        if (w != 0) {
            size_t found_bit = (size_t)__builtin_ctzll(w);
            size_t idx = i * 64 + found_bit;

            if (idx >= start && idx < limit) {
                return idx;
            }
        }
    }

    return BITMAP_INVALID;
}

size_t bitmap_alloc(bitmap_t *bm)
{
    if (bm == NULL || bm->bits == NULL || bm->bit_count == 0) {
        return BITMAP_INVALID;
    }

    size_t start_word = bm->alloc_hint;

    if (start_word >= bm->word_count) {
        start_word = 0;
    }

    size_t idx = find_next_zero_bit(bm, start_word * 64);

    if (idx == BITMAP_INVALID && start_word != 0) {
        idx = find_next_zero_bit(bm, 0);
    }

    if (idx == BITMAP_INVALID) {
        return BITMAP_INVALID;
    }

    bitmap_set(bm, idx);
    bm->alloc_hint = idx / 64;

    return idx;
}

size_t bitmap_alloc_range(bitmap_t *bm, size_t count, size_t align)
{
    if (bm == NULL || bm->bits == NULL || count == 0 || bm->bit_count == 0) {
        return BITMAP_INVALID;
    }

    if (count > bm->bit_count) {
        return BITMAP_INVALID;
    }

    if (align == 0) {
        align = 1;
    }

    /*
     * Bit alignment must be a power of two.
     */
    if ((align & (align - 1)) != 0) {
        align = 1;
    }

    if (count == 1 && align == 1) {
        return bitmap_alloc(bm);
    }

    size_t candidate = 0;

    while (candidate <= bm->bit_count - count) {
        candidate = align_up_size(candidate, align);

        if (candidate == BITMAP_INVALID ||
            candidate > bm->bit_count - count) {
            break;
        }

        size_t free_bit = find_next_zero_bit(bm, candidate);

        if (free_bit == BITMAP_INVALID) {
            break;
        }

        candidate = align_up_size(free_bit, align);

        if (candidate == BITMAP_INVALID ||
            candidate > bm->bit_count - count) {
            break;
        }

        if (bitmap_test_range_free(bm, candidate, count)) {
            bitmap_set_range(bm, candidate, count);
            bm->alloc_hint = candidate / 64;
            return candidate;
        }

        /*
         * If the range is not free, skip past the first used bit in the
         * checked range.
         */
        size_t used = find_next_used_bit(bm, candidate, candidate + count);

        if (used == BITMAP_INVALID) {
            candidate++;
        } else {
            candidate = used + 1;
        }
    }

    return BITMAP_INVALID;
}

void bitmap_free(bitmap_t *bm, size_t bit)
{
    if (bm == NULL || bm->bits == NULL || bit >= bm->bit_count) {
        return;
    }

    bitmap_clear(bm, bit);

    size_t word = bit / 64;

    if (word < bm->alloc_hint) {
        bm->alloc_hint = word;
    }
}

void bitmap_free_range(bitmap_t *bm, size_t start, size_t count)
{
    if (bm == NULL || bm->bits == NULL || count == 0) {
        return;
    }

    if (start >= bm->bit_count) {
        return;
    }

    if (count > bm->bit_count - start) {
        count = bm->bit_count - start;
    }

    bitmap_clear_range(bm, start, count);

    size_t word = start / 64;

    if (word < bm->alloc_hint) {
        bm->alloc_hint = word;
    }
}

size_t bitmap_count_free(const bitmap_t *bm)
{
    if (bm == NULL || bm->bits == NULL) {
        return 0;
    }

    size_t free_bits = 0;

    for (size_t i = 0; i < bm->word_count; i++) {
        free_bits += (size_t)__builtin_popcountll(~bm->bits[i]);
    }

    return free_bits;
}

