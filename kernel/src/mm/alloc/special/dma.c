#include "dma.h"
#include "../physical/pmm.h"
#include "../virtual/vmm.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"

/*
 * P1.4: taki sam problem overflow jak w contiguous.c — patrz komentarz
 * tam. Dodatkowo `frames * ARCH_PAGE_SIZE` (dlugosc mapowania) rowniez
 * jest teraz liczone przez bezpieczny helper zamiast surowego mnozenia.
 */
static bool dma_bytes_to_frames(size_t bytes, size_t *out_frames)
{
	return size_bytes_to_pages_checked(bytes, ARCH_PAGE_SIZE, out_frames);
}

bool dma_alloc_coherent(size_t bytes,
	                    uint64_t zone_max,
	                    uint64_t *out_phys,
	                    void **out_virt)
{
	if (bytes == 0 || out_phys == NULL || out_virt == NULL) {
	    return false;
	}

	if (zone_max == 0) {
	    zone_max = DMA_ZONE_32BIT;
	}

	size_t frames;

	if (!dma_bytes_to_frames(bytes, &frames)) {
	    return false;
	}

	size_t len;

	if (!size_pages_to_bytes_checked(frames, ARCH_PAGE_SIZE, &len)) {
	    return false;
	}

	uint64_t phys = 0;

	if (!pmm_alloc_frames_below(frames, 1, zone_max, &phys)) {
	    return false;
	}

	uint64_t virt = 0;

	if (!vmm_map_device(phys, len, &virt)) {
	    pmm_free_frames(phys, frames);
	    return false;
	}

	*out_phys = phys;
	*out_virt = (void *)(uintptr_t)virt;

	return true;
}

void dma_free_coherent(uint64_t phys, void *virt, size_t bytes)
{
	if (bytes == 0) {
	    return;
	}

	size_t frames;

	if (!dma_bytes_to_frames(bytes, &frames)) {
	    return;
	}

	size_t len;

	if (!size_pages_to_bytes_checked(frames, ARCH_PAGE_SIZE, &len)) {
	    return;
	}

	if (virt != NULL) {
	    vmm_unmap_device((uint64_t)(uintptr_t)virt, len);
	}

	pmm_free_frames(phys, frames);
}

static void clflush_range(void *addr, size_t len)
{
	uintptr_t start = (uintptr_t)addr & ~(uintptr_t)63;
	uintptr_t end = (uintptr_t)addr + len;

	for (uintptr_t a = start; a < end; a += 64) {
	    __asm__ volatile("clflush (%0)" :: "r"(a) : "memory");
	}

	__asm__ volatile("" ::: "memory");
}

void dma_sync_for_device(void *virt, size_t len)
{
	if (virt == NULL || len == 0) {
	    return;
	}

	clflush_range(virt, len);
}

void dma_sync_for_cpu(void *virt, size_t len)
{
	if (virt == NULL || len == 0) {
	    return;
	}

	clflush_range(virt, len);
}