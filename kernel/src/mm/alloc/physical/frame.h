#ifndef MM_ALLOC_PHYSICAL_FRAME_H
#define MM_ALLOC_PHYSICAL_FRAME_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef uint64_t frame_t;

#define FRAME_INVALID UINT64_MAX

bool frame_init(uint64_t bitmap_phys, size_t bit_count);

bool frame_init_from_memory(void);

bool frame_ready(void);

bool frame_alloc(frame_t *out);
bool frame_alloc_zero(frame_t *out);

bool frame_alloc_contiguous(size_t count,
                            size_t align_frames,
                            frame_t *out);
bool frame_alloc_below(size_t count,
                       size_t align_frames,
                       uint64_t max_phys,
                       frame_t *out);

bool frame_free(frame_t frame);
bool frame_free_contiguous(frame_t start, size_t count);

bool frame_is_valid(frame_t frame);
bool frame_zero(frame_t frame);

void *frame_virt(frame_t frame);
uint64_t frame_phys(frame_t frame);

size_t frame_to_pfn(frame_t frame);
frame_t frame_from_pfn(size_t pfn);

size_t frame_total(void);
size_t frame_allocated(void);
size_t frame_free_count(void);

/*
 * P1 (sekcja 4.1 planu ulepszeń: "strefy DMA32/NORMAL... z osobnymi
 * statystykami oraz polityką preferencji"). Granica strefy DMA32 jest
 * stała (< 4 GiB, standardowa definicja x86), ustalana raz w
 * frame_init()/frame_init_from_memory(). Strefa HIGHMEM nie ma
 * zastosowania w tym jądrze — cała pamięć fizyczna jest bezpośrednio
 * adresowalna przez arch_phys_to_virt() (bez potrzeby tymczasowego
 * mapowania), więc dzielimy tylko na DMA32/NORMAL.
 *
 * Polityka preferencji: `frame_alloc()` (zwykła alokacja bez ograniczeń)
 * PREFERUJE strefę NORMAL (>= 4 GiB), zawijając do DMA32 dopiero gdy
 * NORMAL jest wyczerpana — żeby zwykłe alokacje nie zjadały z czasem
 * całej niskiej pamięci potrzebnej urządzeniom DMA32. `frame_alloc_below()`
 * (używana przez dma.c) jest tym niezależna — zawsze może sięgnąć do
 * DMA32, bo to jej jedyny cel.
 */
size_t frame_zone_dma32_total(void);
size_t frame_zone_dma32_free(void);
size_t frame_zone_normal_total(void);
size_t frame_zone_normal_free(void);

#endif /* MM_ALLOC_PHYSICAL_FRAME_H */