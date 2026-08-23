#include "tlb.h"
#include "tlb_asm.h"

#define CPUID1_PCID    (1u << 17)
#define CPUID7_INVPCID (1u << 10)

#define CR4_PGE   (1ULL << 7)
#define CR4_PCIDE (1ULL << 17)

#define INVPCID_ADDR_IN_PCID 0
#define INVPCID_SINGLE_PCID  1
#define INVPCID_ALL          2
#define INVPCID_ALL_NON_GLOBAL 3

#define TLB_RANGE_THRESHOLD 32

static bool tlb_initialized = false;
static bool has_pcid = false;
static bool has_invpcid = false;

typedef struct invpcid_desc {
	uint64_t pcid;
	uint64_t addr;
} __attribute__((aligned(16))) invpcid_desc_t;

static void cpuid(uint32_t leaf,
	              uint32_t sub,
	              uint32_t *a,
	              uint32_t *b,
	              uint32_t *c,
	              uint32_t *d)
{
	__asm__ volatile(
	    "cpuid"
	    : "=a"(*a), "=b"(*b), "=c"(*c), "=d"(*d)
	    : "a"(leaf), "c"(sub));
}

bool tlb_init(void)
{
	if (tlb_initialized) {
	    return true;
	}

	uint32_t a, b, c, d;

	cpuid(1, 0, &a, &b, &c, &d);
	has_pcid = (c & CPUID1_PCID) != 0;

	cpuid(7, 0, &a, &b, &c, &d);
	has_invpcid = (b & CPUID7_INVPCID) != 0;

	tlb_initialized = true;

	return true;
}

bool tlb_ready(void)
{
	return tlb_initialized;
}

bool tlb_has_pcid(void)
{
	return has_pcid;
}

bool tlb_has_invpcid(void)
{
	return has_invpcid;
}

void tlb_flush_all(void)
{
	if (has_invpcid) {
	    invpcid_desc_t desc = { 0, 0 };
	    tlb_asm_invpcid(INVPCID_ALL, &desc);
	    return;
	}

	tlb_asm_write_cr3(tlb_asm_read_cr3());
}

void tlb_flush_all_including_global(void)
{
	if (has_invpcid) {
	    tlb_flush_all();
	    return;
	}

	uint64_t cr4 = tlb_asm_read_cr4();

	if (cr4 & CR4_PGE) {
	    tlb_asm_write_cr4(cr4 & ~CR4_PGE);
	    tlb_asm_write_cr4(cr4);
	} else {
	    tlb_flush_all();
	}
}

void tlb_flush_page_addr(uint64_t addr)
{
	tlb_asm_invlpg(addr & ~TLB_PAGE_MASK);
}

void tlb_flush_page(const void *addr)
{
	if (addr == NULL) {
	    tlb_flush_all();
	    return;
	}

	tlb_flush_page_addr((uint64_t)(uintptr_t)addr);
}

void tlb_flush_range_addr(uint64_t addr, size_t pages)
{
	if (pages == 0) {
	    return;
	}

	if (pages > TLB_RANGE_THRESHOLD) {
	    tlb_flush_all();
	    return;
	}

	uint64_t a = addr & ~TLB_PAGE_MASK;

	if (pages > (UINT64_MAX - a) / TLB_PAGE_SIZE) {
	    tlb_flush_all();
	    return;
	}

	for (size_t i = 0; i < pages; i++) {
	    tlb_asm_invlpg(a);
	    a += TLB_PAGE_SIZE;
	}
}

void tlb_flush_range(const void *addr, size_t pages)
{
	if (addr == NULL) {
	    tlb_flush_all();
	    return;
	}

	tlb_flush_range_addr((uint64_t)(uintptr_t)addr, pages);
}

void tlb_flush_pcid(uint16_t pcid)
{
	if (has_invpcid) {
	    invpcid_desc_t desc = { pcid, 0 };
	    tlb_asm_invpcid(INVPCID_SINGLE_PCID, &desc);
	    return;
	}

	if (pcid == TLB_PCID_KERNEL) {
	    tlb_asm_write_cr3(tlb_asm_read_cr3());
	}
}

void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr)
{
	if (has_invpcid) {
	    invpcid_desc_t desc = { pcid, addr & ~TLB_PAGE_MASK };
	    tlb_asm_invpcid(INVPCID_ADDR_IN_PCID, &desc);
	    return;
	}

	tlb_asm_invlpg(addr & ~TLB_PAGE_MASK);
}

void tlb_wbinvd(void)
{
	tlb_asm_wbinvd();
}

void tlb_clflush(uint64_t addr)
{
	tlb_asm_clflush(addr);
}