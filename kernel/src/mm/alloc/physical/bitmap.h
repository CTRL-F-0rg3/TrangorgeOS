#ifndef MM_ALLOC_PHYSICAL_BITMAP_H
#define MM_ALLOC_PHYSICAL_BITMAP_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define BITMAP_INVALID ((size_t)-1)

typedef struct bitmap {
    uint64_t *bits;

    /*
     * Number of bits in the bitmap.
     */
    size_t bit_count;

    /*
     * Number of 64-bit words.
     */
    size_t word_count;

    /*
     * Hint for first-fit allocation. The word index from which to start
     * searching.
     */
    size_t alloc_hint;
} bitmap_t;

size_t bitmap_words_for_bits(size_t bit_count);
size_t bitmap_bytes_for_bits(size_t bit_count);

void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count);

bool bitmap_init_phys(bitmap_t *bm, uint64_t storage_phys, size_t bit_count);

void bitmap_fill(bitmap_t *bm, bool value);

void bitmap_set(bitmap_t *bm, size_t bit);
void bitmap_clear(bitmap_t *bm, size_t bit);
bool bitmap_test(const bitmap_t *bm, size_t bit);

void bitmap_fill(bitmap_t *bm, bool value);

void bitmap_set(bitmap_t *bm, size_t bit);
void bitmap_clear(bitmap_t *bm, size_t bit);
bool bitmap_test(const bitmap_t *bm, size_t bit);

void bitmap_set_range(bitmap_t *bm, size_t start, size_t count);
void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count);

bool bitmap_test_range_free(const bitmap_t *bm, size_t start, size_t count);

size_t bitmap_alloc(bitmap_t *bm);


size_t bitmap_alloc_from(bitmap_t *bm, size_t min_bit);

size_t bitmap_alloc_range(bitmap_t *bm, size_t count, size_t align);

void bitmap_free(bitmap_t *bm, size_t bit);
void bitmap_free_range(bitmap_t *bm, size_t start, size_t count);

size_t bitmap_count_free(const bitmap_t *bm);

#endif 