#ifndef MM_ALLOC_VIRTUAL_VMM_H
#define MM_ALLOC_VIRTUAL_VMM_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define VMM_INVALID UINT64_MAX

#define VMM_FLAG_WRITE  (1u << 0)
#define VMM_FLAG_USER   (1u << 1)
#define VMM_FLAG_NX     (1u << 2)
#define VMM_FLAG_DEVICE (1u << 3)
#define VMM_FLAG_ZERO   (1u << 4)

#define VMM_KERNEL_RW (VMM_FLAG_WRITE | VMM_FLAG_NX)

bool vmm_init(void);
bool vmm_ready(void);

bool vmm_alloc(size_t bytes, uint32_t flags, uint64_t *out_virt);
bool vmm_alloc_aligned(size_t bytes, size_t align, uint32_t flags, uint64_t *out_virt);
bool vmm_map_device(uint64_t phys, size_t len, uint64_t *out_virt);

bool vmm_free(uint64_t virt, size_t bytes);
bool vmm_unmap_device(uint64_t virt, size_t len);

uint64_t vmm_translate(uint64_t virt);

size_t vmm_stat_total_pages(void);
size_t vmm_stat_free_pages(void);
size_t vmm_stat_allocated_pages(void);
uint64_t vmm_stat_total_bytes(void);
uint64_t vmm_stat_free_bytes(void);

void vmm_dump(void);

#endif