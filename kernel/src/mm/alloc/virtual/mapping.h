#ifndef MM_ALLOC_VIRTUAL_MAPPING_H
#define MM_ALLOC_VIRTUAL_MAPPING_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define MAPPING_KERNEL 0

typedef uint64_t mapping_space_t;

void mapping_init(void);
mapping_space_t mapping_kernel_space(void);

bool mapping_map_range(mapping_space_t space,
                       uint64_t virt,
                       uint64_t phys,
                       size_t len,
                       uint64_t pte_flags);

bool mapping_unmap_range(mapping_space_t space,
                         uint64_t virt,
                         size_t len);

bool mapping_protect_range(mapping_space_t space,
                           uint64_t virt,
                           size_t len,
                           uint64_t pte_flags);

uint64_t mapping_translate(mapping_space_t space, uint64_t virt);
bool mapping_is_mapped(mapping_space_t space, uint64_t virt);

bool mapping_copy_range(mapping_space_t dst,
                        uint64_t dst_virt,
                        mapping_space_t src,
                        uint64_t src_virt,
                        size_t len);

#endif