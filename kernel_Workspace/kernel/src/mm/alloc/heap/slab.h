#ifndef MM_ALLOC_HEAP_SLAB_H
#define MM_ALLOC_HEAP_SLAB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define SLAB_MAX_SIZE 2048

bool slab_init(uint64_t base, size_t size);
bool slab_ready(void);

void *slab_alloc(size_t size);
void slab_free(void *ptr);

size_t slab_usable_size(void *ptr);

size_t slab_stat_used_bytes(void);
size_t slab_stat_free_bytes(void);

void slab_dump(void);

#endif