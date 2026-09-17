#ifndef ARCH_AARCH64_MEMORY_H
#define ARCH_AARCH64_MEMORY_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define ARCH_PAGE_SIZE 4096ULL
#define ARCH_DIRECT_MAP_BASE 0xFFFF888000000000ULL

typedef struct arch_raw_mem_entry {
    uint64_t base;
    uint64_t len;
    uint32_t typ;
    uint32_t reserved;
} arch_raw_mem_entry_t;

void arch_memory_init(const arch_raw_mem_entry_t *entries,
                      size_t count,
                      uint64_t kernel_phys_start,
                      uint64_t kernel_phys_end,
                      uint64_t initrd_phys_start,
                      uint64_t initrd_phys_end);

bool arch_memory_ready(void);

bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out);
void arch_memory_reserve_range(uint64_t base, uint64_t len);

bool arch_is_page_aligned(uint64_t a);
uint64_t arch_page_align_up(uint64_t a);
uint64_t arch_page_align_down(uint64_t a);

void *arch_phys_to_virt(uint64_t phys);
uint64_t arch_virt_to_phys(void *virt);

uint64_t arch_memory_max_phys(void);
void arch_memory_dump(void);

#endif