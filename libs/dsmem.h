#ifndef DSMEM_H
#define DSMEM_H
#include "dsabi.h"

static inline uint64_t ds_alloc_pages(uint32_t pages)
{
    return ds_call(SVC_SYS, OP_ALLOC, pages, 0, 0);
}

static inline uint64_t ds_free_pages(uint64_t va)
{
    return ds_call(SVC_SYS, OP_FREE, va, 0, 0);
}

static inline uint64_t ds_map_mmio(uint64_t phys, uint64_t len, uint64_t va)
{
    return ds_call(SVC_SYS, OP_MAPMMIO, phys, len, va);
}

static inline uint64_t ds_page_phys(uint64_t va)
{
    return ds_call(SVC_SYS, OP_PAGEPHYS, va, 0, 0);
}

#endif