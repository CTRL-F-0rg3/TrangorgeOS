#ifndef ARCH_AARCH64_PAGING_H
#define ARCH_AARCH64_PAGING_H

#include <stdint.h>
#include <stdbool.h>

#define PTE_PRESENT      (1ULL << 0)
#define PTE_WRITABLE     (1ULL << 1)
#define PTE_USER         (1ULL << 2)
#define PTE_NX           (1ULL << 3)
#define PTE_DEVICE       (1ULL << 4)
#define PTE_CACHE_DISABLE (1ULL << 5)
#define PTE_LARGE        (1ULL << 6)

#define PAGING_ADDR_MASK 0x0000FFFFFFFFF000ULL

void paging_init(uint64_t boot_phys_offset);

bool paging_enable_nx(void);

uint64_t paging_read_cr3(void);
void paging_write_cr3(uint64_t pml4_phys);

bool paging_map_page_in(uint64_t pml4, uint64_t virt, uint64_t phys, uint64_t flags);
bool paging_unmap_page_in(uint64_t pml4, uint64_t virt);
bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags);
bool paging_is_mapped_in(uint64_t pml4, uint64_t virt);
uint64_t paging_translate_in(uint64_t pml4, uint64_t virt);

#endif