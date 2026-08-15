#ifndef ARCH_X86_64_PAGING_H
#define ARCH_X86_64_PAGING_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define PAGING_PAGE_SIZE       4096ULL
#define PAGING_PAGE_MASK       (PAGING_PAGE_SIZE - 1ULL)

#define PAGING_2M_PAGE_SIZE    (2ULL * 1024ULL * 1024ULL)
#define PAGING_2M_PAGE_MASK    (PAGING_2M_PAGE_SIZE - 1ULL)

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

void paging_set_boot_phys_offset(uint64_t phys_offset);

bool paging_boot_phys_offset_valid(void);

void paging_init_direct_map(void);

void paging_init(uint64_t boot_phys_offset);

uint64_t paging_read_cr3(void);
void paging_flush_tlb_all(void);
void paging_flush_page(uint64_t addr);

bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags);

bool paging_map_range(uint64_t virt, uint64_t phys, uint64_t len, uint64_t flags);

#endif /* ARCH_X86_64_PAGING_H */