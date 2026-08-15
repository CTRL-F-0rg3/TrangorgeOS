#ifndef MM_ALLOC_HEAP_HEAP_H
#define MM_ALLOC_HEAP_HEAP_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool heap_init(void);
bool heap_ready(void);

void *heap_alloc(size_t size);
void *heap_alloc_aligned(size_t size, size_t align);
void *heap_zalloc(size_t size);

void heap_free(void *ptr);

size_t heap_usable_size(void *ptr);

void heap_dump(void);

#endif