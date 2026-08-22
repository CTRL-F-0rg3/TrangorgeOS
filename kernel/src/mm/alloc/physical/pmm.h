#ifndef MM_ALLOC_PHYSICAL_PMM_H
#define MM_ALLOC_PHYSICAL_PMM_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define PMM_INVALID_FRAME UINT64_MAX

bool pmm_init(void);

bool pmm_ready(void);

bool pmm_alloc_frame(uint64_t *out_phys);

bool pmm_alloc_zero_frame(uint64_t *out_phys);

bool pmm_alloc_frames(size_t count, uint64_t *out_phys);

bool pmm_alloc_frames_aligned(size_t count,
                              size_t align_frames,
                              uint64_t *out_phys);
bool pmm_alloc_bytes(size_t bytes, uint64_t *out_phys);
bool pmm_alloc_zero_bytes(size_t bytes, uint64_t *out_phys);
bool pmm_alloc_contiguous_bytes(size_t bytes,
                                size_t align_bytes,
                                uint64_t *out_phys);
bool pmm_free_frame(uint64_t phys);
bool pmm_alloc_frames_below(size_t count,
                           size_t align_frames,
                           uint64_t max_phys,
                           uint64_t *out_phys);
bool pmm_free_frames(uint64_t phys, size_t count);
bool pmm_free_bytes(uint64_t phys, size_t bytes);
size_t pmm_stat_total_frames(void);
size_t pmm_stat_free_frames(void);
size_t pmm_stat_allocated_frames(void);

uint64_t pmm_stat_total_bytes(void);
uint64_t pmm_stat_free_bytes(void);

/*
 * P1 (sekcja 4.1 planu ulepszeń): statystyki per strefa fizyczna.
 * DMA32 = ramki z adresem fizycznym < 4 GiB; NORMAL = reszta. Zobacz
 * komentarz przy frame_zone_* w alloc/physical/frame.h po pełne
 * uzasadnienie (w tym dlaczego nie ma osobnej strefy HIGHMEM w tym
 * jądrze).
 */
size_t pmm_stat_dma32_total_frames(void);
size_t pmm_stat_dma32_free_frames(void);
size_t pmm_stat_normal_total_frames(void);
size_t pmm_stat_normal_free_frames(void);

void pmm_dump(void);

#ifdef PMM_DEBUG
bool pmm_self_test(void);
#endif

#endif /* MM_ALLOC_PHYSICAL_PMM_H */