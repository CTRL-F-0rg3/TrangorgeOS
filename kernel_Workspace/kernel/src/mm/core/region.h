#ifndef MM_CORE_REGION_H
#define MM_CORE_REGION_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef struct region {
    uint64_t start;
    uint64_t end;
} region_t;

region_t region_make(uint64_t start, uint64_t len);
bool region_valid(region_t r);
uint64_t region_len(region_t r);

bool region_contains(region_t r, uint64_t addr);
bool region_overlaps(region_t a, region_t b);
region_t region_intersect(region_t a, region_t b);

size_t region_subtract(region_t r,
                       region_t cut,
                       region_t *out,
                       size_t max_out);

#endif