#ifndef MM_ALLOC_PHYSICAL_FRAME_H
#define MM_ALLOC_PHYSICAL_FRAME_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef uint64_t frame_t;

#define FRAME_INVALID UINT64_MAX

bool frame_init(uint64_t bitmap_phys, size_t bit_count);

bool frame_init_from_memory(void);

bool frame_ready(void);

bool frame_alloc(frame_t *out);
bool frame_alloc_zero(frame_t *out);

bool frame_alloc_contiguous(size_t count,
                            size_t align_frames,
                            frame_t *out);
bool frame_alloc_below(size_t count,
                       size_t align_frames,
                       uint64_t max_phys,
                       frame_t *out);

bool frame_free(frame_t frame);
bool frame_free_contiguous(frame_t start, size_t count);

bool frame_is_valid(frame_t frame);
bool frame_zero(frame_t frame);

void *frame_virt(frame_t frame);
uint64_t frame_phys(frame_t frame);

size_t frame_to_pfn(frame_t frame);
frame_t frame_from_pfn(size_t pfn);

size_t frame_total(void);
size_t frame_allocated(void);
size_t frame_free_count(void);

#endif /* MM_ALLOC_PHYSICAL_FRAME_H */