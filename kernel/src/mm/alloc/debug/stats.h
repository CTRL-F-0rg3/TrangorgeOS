#ifndef MM_ALLOC_DEBUG_STATS_H
#define MM_ALLOC_DEBUG_STATS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

void alloc_stats_note_alloc(size_t size);
void alloc_stats_note_free(size_t size);

size_t alloc_stats_live_count(void);
size_t alloc_stats_live_bytes(void);
size_t alloc_stats_peak_bytes(void);
size_t alloc_stats_total_allocs(void);
size_t alloc_stats_total_frees(void);

void alloc_stats_dump(void);

#endif