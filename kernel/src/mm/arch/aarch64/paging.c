#include "paging.h"
#include "memory.h"
#include "tlb.h"

#define L0_SHIFT 39
#define L1_SHIFT 30
#define L2_SHIFT 21
#define L3_SHIFT 12
#define IDX(va, sh) (((va) >> (sh)) & 0x1FF)

#define HW_VALID  (1ULL << 0)
#define HW_TABLE  (3ULL << 0)
#define HW_PAGE   (3ULL << 0)
#define HW_BLOCK  (1ULL << 0)
#define HW_AF     (1ULL << 10)
#define HW_SH     (3ULL << 8)
#define HW_UXN    (1ULL << 54)
#define HW_PXN    (1ULL << 53)

#define ATTR_DEV  (0ULL << 2)
#define ATTR_NC   (2ULL << 2)
#define ATTR_WB   (1ULL << 2)

static uint64_t early_base = 0;
static uint64_t kernel_l0 = 0;

static void *early_ptr(uint64_t phys)
{
	return (void *)(early_base + phys);
}

void *arch_phys_to_virt(uint64_t phys)
{
	return (void *)(ARCH_DIRECT_MAP_BASE + phys);
}

static uint64_t *tbl(uint64_t phys)
{
	return (uint64_t *)arch_phys_to_virt(phys);
}

static uint64_t *tbl_early(uint64_t phys)
{
	return (uint64_t *)early_ptr(phys);
}

static uint64_t hw_bits(uint64_t flags, int level)
{
	uint64_t e;

	if (level < 3 && !(flags & PTE_LARGE)) {
	    return HW_TABLE;
	}

	e = (level == 3) ? HW_PAGE : HW_BLOCK;

	uint64_t ap = 0;

	if (flags & PTE_USER) {
	    ap |= 1;
	}

	if (!(flags & PTE_WRITABLE)) {
	    ap |= 2;
	}

	e |= ap << 6;
	e |= HW_AF | HW_SH;

	if (flags & PTE_DEVICE) {
	    e |= ATTR_DEV;
	} else if (flags & PTE_CACHE_DISABLE) {
	    e |= ATTR_NC;
	} else {
	    e |= ATTR_WB;
	}

	if (flags & PTE_NX) {
	    e |= HW_UXN;
	}

	if (flags & PTE_USER) {
	    e |= HW_PXN;
	}

	return e;
}

static uint64_t alloc_table_early(void)
{
	uint64_t phys = 0;

	arch_memory_boot_alloc(4096, 4096, &phys);

	uint64_t *t = early_ptr(phys);

	for (int i = 0; i < 512; i++) {
	    t[i] = 0;
	}

	return phys;
}

static uint64_t ensure_table(uint64_t *entry, bool early)
{
	if (*entry & HW_VALID) {
	    return *entry & PAGING_ADDR_MASK;
	}

	uint64_t phys;

	if (early) {
	    phys = alloc_table_early();
	} else {
	    arch_memory_boot_alloc(4096, 4096, &phys);

	    uint64_t *t = tbl(phys);

	    for (int i = 0; i < 512; i++) {
	        t[i] = 0;
	    }
	}

	*entry = phys | HW_TABLE;

	return phys;
}

bool paging_map_page_in(uint64_t pml4, uint64_t virt, uint64_t phys, uint64_t flags)
{
	uint64_t *l0 = tbl(pml4);

	uint64_t l1 = ensure_table(&l0[IDX(virt, L0_SHIFT)], false);
	uint64_t *l1t = tbl(l1);

	if (flags & PTE_LARGE) {
	    l1t[IDX(virt, L1_SHIFT)] = (phys & 0x0000FFFFFFE00000ULL)
	                             | hw_bits(flags, 1);
	    tlb_flush_page_addr(virt);
	    return true;
	}

	uint64_t l2 = ensure_table(&l1t[IDX(virt, L1_SHIFT)], false);
	uint64_t *l2t = tbl(l2);

	uint64_t l3 = ensure_table(&l2t[IDX(virt, L2_SHIFT)], false);
	uint64_t *l3t = tbl(l3);

	l3t[IDX(virt, L3_SHIFT)] = (phys & PAGING_ADDR_MASK) | hw_bits(flags, 3);

	tlb_flush_page_addr(virt);

	return true;
}

bool paging_unmap_page_in(uint64_t pml4, uint64_t virt)
{
	uint64_t *l0 = tbl(pml4);

	if (!(l0[IDX(virt, L0_SHIFT)] & HW_VALID)) {
	    return false;
	}

	uint64_t *l1t = tbl(l0[IDX(virt, L0_SHIFT)] & PAGING_ADDR_MASK);

	if (!(l1t[IDX(virt, L1_SHIFT)] & HW_VALID)) {
	    return false;
	}

	if ((l1t[IDX(virt, L1_SHIFT)] & 3) == HW_BLOCK) {
	    l1t[IDX(virt, L1_SHIFT)] = 0;
	    tlb_flush_page_addr(virt);
	    return true;
	}

	uint64_t *l2t = tbl(l1t[IDX(virt, L1_SHIFT)] & PAGING_ADDR_MASK);

	if (!(l2t[IDX(virt, L2_SHIFT)] & HW_VALID)) {
	    return false;
	}

	if ((l2t[IDX(virt, L2_SHIFT)] & 3) == HW_BLOCK) {
	    l2t[IDX(virt, L2_SHIFT)] = 0;
	    tlb_flush_page_addr(virt);
	    return true;
	}

	uint64_t *l3t = tbl(l2t[IDX(virt, L2_SHIFT)] & PAGING_ADDR_MASK);

	if (!(l3t[IDX(virt, L3_SHIFT)] & HW_VALID)) {
	    return false;
	}

	l3t[IDX(virt, L3_SHIFT)] = 0;
	tlb_flush_page_addr(virt);

	return true;
}

uint64_t paging_translate_in(uint64_t pml4, uint64_t virt)
{
	uint64_t *l0 = tbl(pml4);

	if (!(l0[IDX(virt, L0_SHIFT)] & HW_VALID)) {
	    return UINT64_MAX;
	}

	uint64_t *l1t = tbl(l0[IDX(virt, L0_SHIFT)] & PAGING_ADDR_MASK);
	uint64_t e1 = l1t[IDX(virt, L1_SHIFT)];

	if (!(e1 & HW_VALID)) {
	    return UINT64_MAX;
	}

	if ((e1 & 3) == HW_BLOCK) {
	    return (e1 & 0x0000FFFFFFE00000ULL) | (virt & 0x1FFFFFULL);
	}

	uint64_t *l2t = tbl(e1 & PAGING_ADDR_MASK);
	uint64_t e2 = l2t[IDX(virt, L2_SHIFT)];

	if (!(e2 & HW_VALID)) {
	    return UINT64_MAX;
	}

	if ((e2 & 3) == HW_BLOCK) {
	    return (e2 & 0x0000FFFFFFFFF000ULL) | (virt & 0xFFFFFULL);
	}

	uint64_t *l3t = tbl(e2 & PAGING_ADDR_MASK);
	uint64_t e3 = l3t[IDX(virt, L3_SHIFT)];

	if (!(e3 & HW_VALID)) {
	    return UINT64_MAX;
	}

	return (e3 & PAGING_ADDR_MASK) | (virt & 0xFFFULL);
}

bool paging_is_mapped_in(uint64_t pml4, uint64_t virt)
{
	return paging_translate_in(pml4, virt) != UINT64_MAX;
}

bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags)
{
	uint64_t phys = paging_translate_in(pml4, virt);

	if (phys == UINT64_MAX) {
	    return false;
	}

	uint64_t *l0 = tbl(pml4);
	uint64_t *l1t = tbl(l0[IDX(virt, L0_SHIFT)] & PAGING_ADDR_MASK);
	uint64_t e1 = l1t[IDX(virt, L1_SHIFT)];
	uint64_t *l2t = tbl(e1 & PAGING_ADDR_MASK);
	uint64_t e2 = l2t[IDX(virt, L2_SHIFT)];
	uint64_t *l3t = tbl(e2 & PAGING_ADDR_MASK);

	l3t[IDX(virt, L3_SHIFT)] = (phys & PAGING_ADDR_MASK) | hw_bits(flags, 3);

	tlb_flush_page_addr(virt);

	return true;
}

void paging_init(uint64_t boot_phys_offset)
{
	early_base = boot_phys_offset;

	uint64_t cur;

	__asm__ volatile("mrs %0, ttbr1_el1" : "=r"(cur));

	uint64_t *boot_l0 = tbl_early(cur & PAGING_ADDR_MASK);

	kernel_l0 = alloc_table_early();

	uint64_t *l0 = tbl_early(kernel_l0);

	for (int i = 0; i < 512; i++) {
	    l0[i] = boot_l0[i];
	}

	uint64_t max_phys = arch_memory_max_phys();

	uint64_t dv = ARCH_DIRECT_MAP_BASE;

	uint64_t l1 = ensure_table(&l0[IDX(dv, L0_SHIFT)], true);
	uint64_t *l1t = tbl_early(l1);
	uint64_t l2 = ensure_table(&l1t[IDX(dv, L1_SHIFT)], true);
	uint64_t *l2t = tbl_early(l2);

	for (uint64_t pa = 0; pa < max_phys; pa += 0x200000) {
	    uint64_t idx = ((dv + pa) >> L2_SHIFT) & 0x1FF;

	    if (idx == 0 && pa >= 0x200000 * 512) {
	        break;
	    }

	    l2t[idx] = pa | HW_BLOCK | HW_AF | HW_SH | ATTR_WB | HW_PXN;
	}

	uint64_t mair = 0x00ULL | (0xFFULL << 8) | (0x44ULL << 16);

	__asm__ volatile("msr mair_el1, %0" :: "r"(mair));

	uint64_t tcr = (16ULL << 0)
	             | (16ULL << 16)
	             | (0b00ULL << 14)
	             | (0b10ULL << 30)
	             | (0b01ULL << 8)
	             | (0b01ULL << 10)
	             | (0b11ULL << 12)
	             | (0b01ULL << 24)
	             | (0b01ULL << 26)
	             | (0b11ULL << 28)
	             | (0b010ULL << 32);

	__asm__ volatile("msr tcr_el1, %0" :: "r"(tcr));

	__asm__ volatile("msr ttbr0_el1, %0" :: "r"(kernel_l0));
	__asm__ volatile("msr ttbr1_el1, %0" :: "r"(kernel_l0));

	__asm__ volatile("isb");
	__asm__ volatile("tlbi vmalle1");
	__asm__ volatile("dsb sy");
	__asm__ volatile("isb");
}

bool paging_enable_nx(void)
{
	return true;
}

uint64_t paging_read_cr3(void)
{
	uint64_t v;

	__asm__ volatile("mrs %0, ttbr1_el1" : "=r"(v));

	return v & PAGING_ADDR_MASK;
}

void paging_write_cr3(uint64_t pml4_phys)
{
	__asm__ volatile("msr ttbr0_el1, %0" :: "r"(pml4_phys));
	__asm__ volatile("msr ttbr1_el1, %0" :: "r"(pml4_phys));
	__asm__ volatile("isb");

	tlb_flush_all();
}