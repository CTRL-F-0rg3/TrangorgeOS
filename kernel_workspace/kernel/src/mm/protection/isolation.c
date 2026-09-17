#include "isolation.h"
#include "../paging/pml.h"
#include "../arch/x86_64/paging.h"

extern void kprintf(const char *fmt, ...);

static void cpuid(uint32_t leaf,
	              uint32_t subleaf,
	              uint32_t *a,
	              uint32_t *b,
	              uint32_t *c,
	              uint32_t *d)
{
	__asm__ volatile(
	    "cpuid"
	    : "=a"(*a), "=b"(*b), "=c"(*c), "=d"(*d)
	    : "a"(leaf), "c"(subleaf));
}

static uint64_t read_cr4(void)
{
	uint64_t v;

	__asm__ volatile("mov %%cr4, %0" : "=r"(v));

	return v;
}

static void write_cr4(uint64_t v)
{
	__asm__ volatile("mov %0, %%cr4" :: "r"(v) : "memory");
}

bool isolation_has_smep(void)
{
	uint32_t a, b, c, d;

	cpuid(7, 0, &a, &b, &c, &d);

	return (b >> 7) & 1;
}

bool isolation_has_smap(void)
{
	uint32_t a, b, c, d;

	cpuid(7, 0, &a, &b, &c, &d);

	return (b >> 20) & 1;
}

bool isolation_enable_smep(void)
{
	if (!isolation_has_smep()) {
	    return false;
	}

	write_cr4(read_cr4() | (1ULL << 20));

	return true;
}

bool isolation_enable_smap(void)
{
	if (!isolation_has_smap()) {
	    return false;
	}

	write_cr4(read_cr4() | (1ULL << 21));

	return true;
}

static size_t audit_tables(uint64_t table_phys, int level, size_t violations)
{
	uint64_t *t = pml_table_ptr(table_phys);

	for (size_t i = 0; i < PML_ENTRIES; i++) {
	    uint64_t e = t[i];

	    if (!pml_entry_present(e)) {
	        continue;
	    }

	    if (e & PTE_USER) {
	        violations++;
	    }

	    if (!pml_entry_large(e) && level > PML_LEVEL_PT) {
	        violations =
	            audit_tables(pml_entry_addr(e), level - 1, violations);
	    }
	}

	return violations;
}

size_t isolation_audit_kernel(void)
{
	uint64_t pml4 = paging_read_cr3();
	uint64_t *t = pml_table_ptr(pml4);

	size_t violations = 0;

	for (size_t i = 256; i < 512; i++) {
	    uint64_t e = t[i];

	    if (!pml_entry_present(e)) {
	        continue;
	    }

	    if (e & PTE_USER) {
	        violations++;
	    }

	    if (!pml_entry_large(e)) {
	        violations =
	            audit_tables(pml_entry_addr(e), PML_LEVEL_PDPT, violations);
	    }
	}

	return violations;
}

bool isolation_init(void)
{
	paging_enable_nx();
	isolation_enable_smep();
	isolation_enable_smap();

	return true;
}