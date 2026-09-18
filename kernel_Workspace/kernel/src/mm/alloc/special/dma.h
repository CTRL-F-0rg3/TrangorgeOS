#ifndef MM_ALLOC_SPECIAL_DMA_H
#define MM_ALLOC_SPECIAL_DMA_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define DMA_ZONE_32BIT 0xFFFFFFFFULL
#define DMA_ZONE_64BIT UINT64_MAX

bool dma_alloc_coherent(size_t bytes,
                        uint64_t zone_max,
                        uint64_t *out_phys,
                        void **out_virt);

void dma_free_coherent(uint64_t phys, void *virt, size_t bytes);

void dma_sync_for_device(void *virt, size_t len);
void dma_sync_for_cpu(void *virt, size_t len);

#endif