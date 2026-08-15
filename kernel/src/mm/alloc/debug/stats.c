#include "stats.h"

extern void kprintf(const char *fmt, ...);

static size_t stat_live_count = 0;
static size_t stat_live_bytes = 0;
static size_t stat_peak_bytes = 0;
static size_t stat_total_allocs = 0;
static size_t stat_total_frees = 0;

void alloc_stats_note_alloc(size_t size)
{
    stat_live_count++;
    stat_live_bytes += size;
    stat_total_allocs++;

    if (stat_live_bytes > stat_peak_bytes) {
        stat_peak_bytes = stat_live_bytes;
    }
}

void alloc_stats_note_free(size_t size)
{
    if (stat_live_count > 0) {
        stat_live_count--;
    }

    if (stat_live_bytes >= size) {
        stat_live_bytes -= size;
    } else {
        stat_live_bytes = 0;
    }

    stat_total_frees++;
}

size_t alloc_stats_live_count(void)
{
    return stat_live_count;
}

size_t alloc_stats_live_bytes(void)
{
    return stat_live_bytes;
}

size_t alloc_stats_peak_bytes(void)
{
    return stat_peak_bytes;
}

size_t alloc_stats_total_allocs(void)
{
    return stat_total_allocs;
}

size_t alloc_stats_total_frees(void)
{
    return stat_total_frees;
}

void alloc_stats_dump(void)
{
    kprintf("ALLOC STATS:\n");
    kprintf("  live: %llu objs, %llu B\n",
            (unsigned long long)stat_live_count,
            (unsigned long long)stat_live_bytes);
    kprintf("  peak: %llu B\n",
            (unsigned long long)stat_peak_bytes);
    kprintf("  total allocs: %llu\n",
            (unsigned long long)stat_total_allocs);
    kprintf("  total frees: %llu\n",
            (unsigned long long)stat_total_frees);
}