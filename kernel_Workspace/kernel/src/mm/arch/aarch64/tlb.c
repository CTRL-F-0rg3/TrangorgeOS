#include "tlb.h"

static bool tlb_ready_flag = false;

bool tlb_init(void)
{
	tlb_ready_flag = true;
	return true;
}

bool tlb_ready(void)
{
	return tlb_ready_flag;
}

void tlb_flush_all(void)
{
	__asm__ volatile("dsb sy");
	__asm__ volatile("tlbi vmalle1");
	__asm__ volatile("dsb sy");
	__asm__ volatile("isb");
}

void tlb_flush_all_including_global(void)
{
	tlb_flush_all();
}

void tlb_flush_page_addr(uint64_t addr)
{
	__asm__ volatile("dsb ishst");
	__asm__ volatile("tlbi vaae1is, %0" :: "r"(addr >> 12));
	__asm__ volatile("dsb ish");
	__asm__ volatile("isb");
}

void tlb_flush_page(const void *addr)
{
	tlb_flush_page_addr((uint64_t)(uintptr_t)addr);
}

void tlb_flush_range_addr(uint64_t addr, size_t pages)
{
	if (pages > 32) {
	    tlb_flush_all();
	    return;
	}

	for (size_t i = 0; i < pages; i++) {
	    tlb_flush_page_addr(addr + i * 4096);
	}
}

void tlb_flush_range(const void *addr, size_t pages)
{
	tlb_flush_range_addr((uint64_t)(uintptr_t)addr, pages);
}