#ifndef MM_ALLOC_DEBUG_ALLOC_DEBUG_H
#define MM_ALLOC_DEBUG_ALLOC_DEBUG_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

void *dbg_alloc(size_t size);
void dbg_free(void *ptr);

bool dbg_verify(void *ptr);
size_t dbg_usable_size(void *ptr);
void mm_debug_dump(void);

#endif