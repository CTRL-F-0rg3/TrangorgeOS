#ifndef MM_ALLOC_API_ALLOC_H
#define MM_ALLOC_API_ALLOC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

void *kmalloc(size_t size);
void *kzalloc(size_t size);
void *kcalloc(size_t count, size_t size);
void *krealloc(void *ptr, size_t new_size);
void *kmalloc_aligned(size_t size, size_t align);
void kfree(void *ptr);

void *kalloc_pages(size_t pages);
void kfree_pages(void *ptr, size_t pages);

char *kstrdup(const char *s);

uint64_t kvirt_to_phys(void *ptr);

void kalloc_dump(void);

#endif