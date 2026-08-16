#ifndef ARCH_X86_64_PAGING_H
#define ARCH_X86_64_PAGING_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define PAGING_PAGE_SIZE       4096ULL
#define PAGING_PAGE_MASK       (PAGING_PAGE_SIZE - 1ULL)

#define PAGING_2M_PAGE_SIZE    (2ULL * 1024ULL * 1024ULL)
#define PAGING_2M_PAGE_MASK    (PAGING_2M_PAGE_SIZE - 1ULL)

#define PAGING_1G_PAGE_SIZE    (1ULL << 30)
#define PAGING_1G_PAGE_MASK    (PAGING_1G_PAGE_SIZE - 1ULL)

#define PTE_PRESENT           (1ULL << 0)
#define PTE_WRITABLE          (1ULL << 1)
#define PTE_USER              (1ULL << 2)
#define PTE_WRITE_THROUGH     (1ULL << 3)
#define PTE_CACHE_DISABLE     (1ULL << 4)
#define PTE_ACCESSED          (1ULL << 5)
#define PTE_DIRTY             (1ULL << 6)
#define PTE_PAGE_SIZE         (1ULL << 7)
#define PTE_GLOBAL            (1ULL << 8)
#define PTE_NX                (1ULL << 63)

#define PAGING_ADDR_MASK       0x000FFFFFFFFFF000UL

#define PAGING_KERNEL_RW       (PTE_PRESENT | PTE_WRITABLE)
#define PAGING_KERNEL_RO       (PTE_PRESENT)

/* First kernel PML4 index — higher half of the address space. */
#define PAGING_KERNEL_PML4_START 256

void paging_set_boot_phys_offset(uint64_t phys_offset);

bool paging_boot_phys_offset_valid(void);

void paging_init_direct_map(void);

void paging_init(uint64_t boot_phys_offset);

uint64_t paging_read_cr3(void);
void paging_flush_tlb_all(void);
void paging_flush_page(uint64_t addr);

bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags);

bool paging_map_range(uint64_t virt, uint64_t phys, uint64_t len, uint64_t flags);

/* Stage A — operations on the current address space (CR3). */
uint64_t paging_translate(uint64_t virt);
bool paging_is_mapped(uint64_t virt);
bool paging_unmap_page(uint64_t virt);
bool paging_set_flags(uint64_t virt, uint64_t flags);
bool paging_get_flags(uint64_t virt, uint64_t *out_flags);
void paging_enable_nx(void);
bool paging_nx_enabled(void);

/* Stage B — variants with an explicit PML4 (physical PML4 address). */
bool paging_map_page_in(uint64_t pml4,
                        uint64_t virt,
                        uint64_t phys,
                        uint64_t flags);
bool paging_unmap_page_in(uint64_t pml4, uint64_t virt);
uint64_t paging_translate_in(uint64_t pml4, uint64_t virt);
bool paging_is_mapped_in(uint64_t pml4, uint64_t virt);
bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags);
bool paging_get_flags_in(uint64_t pml4, uint64_t virt, uint64_t *out_flags);

/* Stage B — address space management. */
uint64_t paging_create_pml4(void);
void paging_destroy_pml4(uint64_t pml4_phys);
void paging_switch_pml4(uint64_t pml4_phys);

/* Stage C — MMIO mapping (uncached: cache-disable + write-through). */
bool paging_map_mmio(uint64_t virt, uint64_t phys, uint64_t len);

/* Stage D — CR0.WP (for COW). */
void paging_enable_write_protect(void);
void paging_disable_write_protect(void);
bool paging_write_protect_enabled(void);

/* Stage D — CR4.SMEP / SMAP. */
void paging_enable_smep(void);
void paging_enable_smap(void);
bool paging_smep_enabled(void);
bool paging_smap_enabled(void);

/* Stage D — PCID / invpcid. */
bool paging_pcid_supported(void);
void paging_enable_pcid(void);
bool paging_pcid_enabled(void);
void paging_invpcid(uint64_t type, uint64_t pcid, uint64_t addr);

/* Stage D — 5-level paging (LA57) detection. */
bool paging_la57_supported(void);
void paging_assert_4level_paging(void);
void paging_write_cr3(uint64_t pml4_phys);
#endif /* ARCH_X86_64_PAGING_H */
