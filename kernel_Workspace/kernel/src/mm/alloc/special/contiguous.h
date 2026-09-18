#ifndef MM_ALLOC_SPECIAL_CONTIGUOUS_H
#define MM_ALLOC_SPECIAL_CONTIGUOUS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool contig_alloc(size_t bytes,
                  size_t align,
                  uint64_t *out_phys,
                  void **out_virt);

void contig_free(uint64_t phys, size_t bytes);

#endif