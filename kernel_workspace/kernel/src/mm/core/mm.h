#ifndef MM_CORE_MM_H
#define MM_CORE_MM_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#if defined(__x86_64__)
#include "../arch/x86_64/memory.h"
#elif defined(__aarch64__)
#include "../arch/aarch64/memory.h"
#endif

typedef struct mm_boot_params {
    const arch_raw_mem_entry_t *memmap;
    size_t memmap_count;

    uint64_t kernel_phys_start;
    uint64_t kernel_phys_end;

    uint64_t initrd_phys_start;
    uint64_t initrd_phys_end;

    uint64_t boot_phys_offset;
} mm_boot_params_t;

bool mm_init(const mm_boot_params_t *params);
bool mm_ready(void);

uint64_t mm_total_ram(void);
uint64_t mm_free_ram(void);

void mm_dump(void);

#endif