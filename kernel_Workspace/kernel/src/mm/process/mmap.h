#ifndef MM_PROCESS_MMAP_H
#define MM_PROCESS_MMAP_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "address_space.h"

#define MAP_ANONYMOUS (1u << 0)
#define MAP_PRIVATE   (1u << 1)
#define MAP_SHARED    (1u << 2)
#define MAP_FIXED     (1u << 3)

uint64_t mmap(proc_aspace_t *pa,
              uint64_t addr,
              size_t len,
              uint32_t prot,
              uint32_t flags);

bool munmap(proc_aspace_t *pa, uint64_t addr, size_t len);
bool mprotect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);

#endif