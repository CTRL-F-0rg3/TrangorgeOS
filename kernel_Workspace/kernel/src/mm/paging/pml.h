#ifndef MM_PAGING_PML_H
#define MM_PAGING_PML_H

#include <stdint.h>
#include <stdbool.h>

#define PML_ENTRIES 512

#define PML_LEVEL_PT   1
#define PML_LEVEL_PD   2
#define PML_LEVEL_PDPT 3
#define PML_LEVEL_PML4 4

uint64_t pml_level_shift(int level);
uint64_t pml_index(uint64_t virt, int level);

bool pml_entry_present(uint64_t entry);
bool pml_entry_large(uint64_t entry);
uint64_t pml_entry_addr(uint64_t entry);
uint64_t *pml_table_ptr(uint64_t table_phys);

#endif