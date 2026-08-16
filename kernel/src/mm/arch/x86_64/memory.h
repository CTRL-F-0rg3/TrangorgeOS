// #ifdef ARCH_X86_64_MEMORY_H
// #define ARCH_X86_64_MEMORY_H

// #include <stdint.h>
// #include <stddef.h>

// #define PAGE_SIZE 409
#ifndef ARCH_MAX_MEM_REGIONS
#define ARCH_MAX_MEM_REGIONS 256
#endif

#ifndef ARCH_X86_64_MEMORY_H
#define ARCH_X86_64_MEMORY_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define ARCH_PAGE_SHIFT 12
#define ARCH_PAGE_SIZE  (1ULL << ARCH_PAGE_SHIFT)
#define ARCH_PAGE_MASK  (ARCH_PAGE_SIZE - 1ULL)

#define ARCH_DIRECT_MAP_BASE 0xFFFF888000000000UL

#define ARCH_RAW_MEM_USABLE        1u
#define ARCH_RAW_MEM_RESERVED      2u
#define ARCH_RAW_MEM_ACPI_RECLAIM 3u
#define ARCH_RAW_MEM_ACPI_NVS     4u
#define ARCH_RAW_MEM_BAD          5u
#define ARCH_RAW_MEM_BOOTLOADER   0x100u

typedef enum arch_mem_type {
    ARCH_MEM_TYPE_USABLE = 0,
    ARCH_MEM_TYPE_RESERVED,
    ARCH_MEM_TYPE_ACPI_RECLAIM,
    ARCH_MEM_TYPE_BOOTLOADER,
    ARCH_MEM_TYPE_BAD,
} arch_mem_type_t;

typedef struct arch_mem_region {
    uint64_t base;
    uint64_t len;
    arch_mem_type_t type;
} arch_mem_region_t;

typedef struct arch_raw_mem_entry {
    uint64_t base;
    uint64_t len;
    uint32_t type;
    uint32_t reserved;
} arch_raw_mem_entry_t;

typedef struct arch_mem_info {
    arch_mem_region_t regions[ARCH_MAX_MEM_REGIONS];
    size_t count;

    uint64_t total_usable;
    uint64_t max_address;
    uint64_t max_usable_address;

    uint64_t direct_map_base;
} arch_mem_info_t;

void arch_memory_init(const arch_raw_mem_entry_t *entries,
                      size_t count,
                      uint64_t kernel_phys_start,
                      uint64_t kernel_phys_end,
                      uint64_t initrd_phys_start,
                      uint64_t initrd_phys_end);

bool arch_memory_ready(void);

bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out_base);

const arch_mem_info_t *arch_memory_get(void);
size_t arch_memory_regions(const arch_mem_region_t **out);
uint64_t arch_memory_total_usable(void);
bool arch_memory_range_is_usable(uint64_t base, uint64_t len);
void arch_memory_reserve_range(uint64_t base, uint64_t len);
bool arch_memory_find_usable(uint64_t len,
                             uint64_t align,
                             uint64_t *out_base);
void arch_memory_dump(void);

static inline uint64_t arch_page_align_down(uint64_t v)
{
    return v & ~ARCH_PAGE_MASK;
}

static inline uint64_t arch_page_align_up(uint64_t v)
{
    if (v > UINT64_MAX - ARCH_PAGE_MASK) {
        return UINT64_MAX;
    }

    return (v + ARCH_PAGE_MASK) & ~ARCH_PAGE_MASK;
}

static inline bool arch_is_page_aligned(uint64_t v)
{
    return (v & ARCH_PAGE_MASK) == 0;
}

/*
 * Note: these functions only make sense once the direct map has been set up
 * in paging.c.
 */
static inline void *arch_phys_to_virt(uint64_t phys)
{
    return (void *)(uintptr_t)(phys + ARCH_DIRECT_MAP_BASE);
}

static inline uint64_t arch_virt_to_phys(const void *virt)
{
    return (uint64_t)((uintptr_t)virt - ARCH_DIRECT_MAP_BASE);
}

#endif /* ARCH_X86_64_MEMORY_H */