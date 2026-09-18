#include "mapping.h"
#include "../../arch/x86_64/paging.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/range.h"

static uint64_t mapping_kernel_pml4 = 0;
static bool mapping_initialized = false;

static uint64_t resolve_space(mapping_space_t space)
{
	if (space == MAPPING_KERNEL) {
	    return mapping_kernel_pml4;
	}

	return space;
}

void mapping_init(void)
{
	mapping_kernel_pml4 = paging_read_cr3();
	mapping_initialized = true;
}

mapping_space_t mapping_kernel_space(void)
{
	if (!mapping_initialized) {
	    mapping_init();
	}

	return mapping_kernel_pml4;
}


static bool page_range_ok(uint64_t addr, size_t len)
{
	uint64_t start, end;

	if (!range_from_addr_len(addr, (uint64_t)len, ARCH_PAGE_SIZE,
	                         0, UINT64_MAX, false, &start, &end)) {
	    return false;
	}


	return start == addr && end == addr + (uint64_t)len;
}

bool mapping_map_range(mapping_space_t space,
	                   uint64_t virt,
	                   uint64_t phys,
	                   size_t len,
	                   uint64_t pte_flags)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len) || !page_range_ok(phys, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);
	size_t mapped = 0;

	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;
	    uint64_t p = phys + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_map_page_in(pml4, v, p, pte_flags)) {
	        for (size_t j = 0; j < mapped; j++) {
	            paging_unmap_page_in(pml4,
	                                 virt + (uint64_t)j * ARCH_PAGE_SIZE);
	        }

	        return false;
	    }

	    mapped++;
	}

	return true;
}

bool mapping_unmap_range(mapping_space_t space,
	                     uint64_t virt,
	                     size_t len)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

	for (size_t i = 0; i < pages; i++) {
	    paging_unmap_page_in(pml4, virt + (uint64_t)i * ARCH_PAGE_SIZE);
	}

	return true;
}

bool mapping_protect_range(mapping_space_t space,
	                       uint64_t virt,
	                       size_t len,
	                       uint64_t pte_flags)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_is_mapped_in(pml4, v)) {
	        return false;
	    }
	}

	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_set_flags_in(pml4, v, pte_flags)) {

	        return false;
	    }
	}

	return true;
}

uint64_t mapping_translate(mapping_space_t space, uint64_t virt)
{
	return paging_translate_in(resolve_space(space), virt);
}

bool mapping_is_mapped(mapping_space_t space, uint64_t virt)
{
	return paging_is_mapped_in(resolve_space(space), virt);
}

static void copy_page_by_phys(uint64_t dst_phys, uint64_t src_phys)
{
	uint64_t *dst = (uint64_t *)arch_phys_to_virt(dst_phys);
	const uint64_t *src = (const uint64_t *)arch_phys_to_virt(src_phys);

	for (size_t i = 0; i < ARCH_PAGE_SIZE / sizeof(uint64_t); i++) {
	    dst[i] = src[i];
	}

	__asm__ volatile("" ::: "memory");
}

bool mapping_copy_range(mapping_space_t dst,
	                    uint64_t dst_virt,
	                    mapping_space_t src,
	                    uint64_t src_virt,
	                    size_t len)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(dst_virt, len) || !page_range_ok(src_virt, len)) {
	    return false;
	}

	uint64_t dst_pml4 = resolve_space(dst);
	uint64_t src_pml4 = resolve_space(src);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);


	for (size_t i = 0; i < pages; i++) {
	    uint64_t dv = dst_virt + (uint64_t)i * ARCH_PAGE_SIZE;
	    uint64_t sv = src_virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_is_mapped_in(dst_pml4, dv) ||
	        !paging_is_mapped_in(src_pml4, sv)) {
	        return false;
	    }

	    uint64_t dp = paging_translate_in(dst_pml4, dv);
	    uint64_t sp = paging_translate_in(src_pml4, sv);

	    copy_page_by_phys(dp, sp);
	}

	return true;
}