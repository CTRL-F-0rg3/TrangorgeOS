#ifndef MM_ALLOC_HEAP_BUDDY_H
#define MM_ALLOC_HEAP_BUDDY_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef bool (*buddy_map_cb)(uint64_t virt, size_t size);
typedef void (*buddy_unmap_cb)(uint64_t virt, size_t size);

bool buddy_init(uint64_t base,
                size_t size,
                buddy_map_cb map_cb,
                buddy_unmap_cb unmap_cb);

bool buddy_ready(void);

void *buddy_alloc(size_t size);
void *buddy_alloc_aligned(size_t size, size_t align);
void buddy_free(void *ptr);

size_t buddy_block_size(void *ptr);

size_t buddy_stat_used_bytes(void);
size_t buddy_stat_free_bytes(void);

void buddy_dump(void);

#endif