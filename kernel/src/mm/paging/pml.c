#include "pml.h"
#include "../arch/x86_64/paging.h"
#include "../arch/x86_64/memory.h"

uint64_t pml_level_shift(int level)
{
    switch (level) {
    case PML_LEVEL_PT:
        return 12;
    case PML_LEVEL_PD:
        return 21;
    case PML_LEVEL_PDPT:
        return 30;
    default:
        return 39;
    }
}

uint64_t pml_index(uint64_t virt, int level)
{
    return (virt >> pml_level_shift(level)) & 0x1FF;
}

bool pml_entry_present(uint64_t entry)
{
    return (entry & PTE_PRESENT) != 0;
}

bool pml_entry_large(uint64_t entry)
{
    return (entry & PTE_PAGE_SIZE) != 0;
}

uint64_t pml_entry_addr(uint64_t entry)
{
    return entry & PAGING_ADDR_MASK;
}

uint64_t *pml_table_ptr(uint64_t table_phys)
{
    return (uint64_t *)arch_phys_to_virt(table_phys);
}