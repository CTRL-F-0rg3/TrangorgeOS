#ifndef MM_PAGING_PAGING_H
#define MM_PAGING_PAGING_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define PROT_READ   (1u << 0)
#define PROT_WRITE  (1u << 1)
#define PROT_EXEC   (1u << 2)
#define PROT_USER   (1u << 3)
#define PROT_DEVICE (1u << 4)

typedef struct address_space {
    uint64_t pml4_phys;
    bool kernel;
} address_space_t;

bool paging_subsystem_init(void);

address_space_t *paging_aspace_create(void);
void paging_aspace_destroy(address_space_t *as);
void paging_aspace_switch(address_space_t *as);
uint64_t paging_aspace_cr3(const address_space_t *as);

bool paging_aspace_map(address_space_t *as,
                       uint64_t virt,
                       uint64_t phys,
                       size_t len,
                       uint32_t prot);

bool paging_aspace_unmap(address_space_t *as, uint64_t virt, size_t len);

bool paging_aspace_protect(address_space_t *as,
                           uint64_t virt,
                           size_t len,
                           uint32_t prot);

uint64_t paging_aspace_translate(address_space_t *as, uint64_t virt);

bool paging_kernel_map(uint64_t virt, uint64_t phys, size_t len, uint32_t prot);
bool paging_kernel_unmap(uint64_t virt, size_t len);

#endif