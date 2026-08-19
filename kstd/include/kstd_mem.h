#ifndef KSTD_MEM_H
#define KSTD_MEM_H

#include "kstd_types.h"

void *tr_alloc(uint32_t bytes);
void tr_free(void *ptr, uint32_t bytes);
tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va);

#endif