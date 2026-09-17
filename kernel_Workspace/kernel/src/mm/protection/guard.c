#include "guard.h"
#include "../arch/x86_64/memory.h"

uint64_t guard_install(proc_aspace_t *pa, uint64_t addr, size_t len)
{
	return aspace_reserve_at(pa, addr, len, VMA_FLAG_GUARD);
}

bool guard_user_stack(proc_aspace_t *pa)
{
	uint64_t stack_base = aspace_stack_base();

	if (stack_base < ARCH_PAGE_SIZE) {
	    return false;
	}

	return aspace_reserve_at(pa,
	                         stack_base - ARCH_PAGE_SIZE,
	                         ARCH_PAGE_SIZE,
	                         VMA_FLAG_GUARD) != 0;
}