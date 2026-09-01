#ifndef MM_PROCESS_ADDRESS_SPACE_H
#define MM_PROCESS_ADDRESS_SPACE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "../paging/paging.h"

#define VMA_FLAG_ANON    (1u << 0)
#define VMA_FLAG_PRIVATE (1u << 1)
#define VMA_FLAG_SHARED  (1u << 2)
#define VMA_FLAG_GUARD   (1u << 3)

typedef struct vma {
    uint64_t start;
    uint64_t end;
    uint32_t prot;
    uint32_t flags;
    struct vma *next;
} vma_t;

typedef struct proc_aspace {
    address_space_t *as;
    vma_t *vmas;
    uint64_t brk_base;
    uint64_t brk;
    uint64_t brk_max;
} proc_aspace_t;

bool aspace_subsystem_init(void);

proc_aspace_t *aspace_create(void);
void aspace_destroy(proc_aspace_t *pa);

address_space_t *aspace_paging_handle(proc_aspace_t *pa);

uint64_t aspace_map_anon(proc_aspace_t *pa, uint64_t hint, size_t len, uint32_t prot);
uint64_t aspace_map_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);
uint64_t aspace_reserve_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t flags);
bool aspace_unmap(proc_aspace_t *pa, uint64_t addr, size_t len);
bool aspace_protect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);

bool aspace_protect_checked(proc_aspace_t *pa, uint64_t addr, size_t len,
                            uint32_t checked_prot,
                            uint32_t apply_prot,
                            bool (*allowed)(uint32_t old_prot,
                                            uint32_t checked_prot));


vma_t *aspace_vma_find(proc_aspace_t *pa, uint64_t addr);
uint64_t aspace_stack_base(void);

uint64_t aspace_brk(proc_aspace_t *pa, uint64_t new_brk);

#endif