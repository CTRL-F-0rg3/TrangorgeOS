#ifndef MM_ALLOC_DEBUG_LEAK_H
#define MM_ALLOC_DEBUG_LEAK_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool leak_track(void *ptr, size_t size, uint64_t caller);

bool leak_track_id(void *ptr, size_t size, uint64_t caller, uint64_t *out_id);

bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller);
bool leak_contains(void *ptr);
size_t leak_count(void);

void leak_dump(void);

#endif