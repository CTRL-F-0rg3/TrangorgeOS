#include "mm.h"
#include "../arch/x86_64/paging.h"
#include "../alloc/physical/pmm.h"
#include "../alloc/virtual/vmm.h"
#include "../alloc/virtual/page.h"
#include "../alloc/virtual/mapping.h"
#include "../alloc/heap/heap.h"
#include "../alloc/api/alloc.h"
#include "../paging/paging.h"
#include "../protection/isolation.h"
#include "../process/address_space.h"
#include "../cache/cache.h"


extern void kprintf(const char *fmt, ...);


static bool mm_initialized = false;

bool mm_init(const mm_boot_params_t *params)
{
	if (mm_initialized) {
	    return true;
	}

	if (params == NULL ||
	    params->memmap == NULL ||
	    params->memmap_count == 0) {
	    kprintf("mm_init: bad params\n");
	    return false;
	}

	kprintf("mm_init: arch_memory_init...\n");
	arch_memory_init(params->memmap,
	                 params->memmap_count,
	                 params->kernel_phys_start,
	                 params->kernel_phys_end,
	                 params->initrd_phys_start,
	                 params->initrd_phys_end);

	kprintf("mm_init: paging_init...\n");
	paging_init(params->boot_phys_offset);

	kprintf("mm_init: pmm_init...\n");
	if (!pmm_init()) {
	    kprintf("mm_init: pmm_init FAILED\n");
	    return false;
	}

	kprintf("mm_init: vmm_init...\n");
	if (!vmm_init()) {
	    kprintf("mm_init: vmm_init FAILED\n");
	    return false;
	}

	kprintf("mm_init: page_init...\n");
	if (!page_init()) {
	    kprintf("mm_init: page_init FAILED\n");
	    return false;
	}

	mapping_init();

	kprintf("mm_init: heap_init...\n");
	if (!heap_init()) {
	    kprintf("mm_init: heap_init FAILED\n");
	    return false;
	}

	kprintf("mm_init: cache_init...\n");
	if (!cache_init()) {
	    kprintf("mm_init: cache_init FAILED\n");
	    return false;
	}

	kprintf("mm_init: paging_subsystem_init...\n");
	if (!paging_subsystem_init()) {
	    kprintf("mm_init: paging_subsystem_init FAILED\n");
	    return false;
	}

	isolation_init();

	kprintf("mm_init: aspace_subsystem_init...\n");
	if (!aspace_subsystem_init()) {
	    kprintf("mm_init: aspace_subsystem_init FAILED\n");
	    return false;
	}

	mm_initialized = true;

	kprintf("mm_init: OK\n");

	return true;
}

bool mm_ready(void)
{
	return mm_initialized;
}

uint64_t mm_total_ram(void)
{
	return pmm_stat_total_bytes();
}

uint64_t mm_free_ram(void)
{
	return pmm_stat_free_bytes();
}

void mm_dump(void)
{
	kprintf("MM: ready=%d total=%llu MiB free=%llu MiB\n",
	        (int)mm_initialized,
	        (unsigned long long)(mm_total_ram() >> 20),
	        (unsigned long long)(mm_free_ram() >> 20));

	kalloc_dump();
}
