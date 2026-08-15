#include "mmap.h"

uint64_t mmap(proc_aspace_t *pa,
              uint64_t addr,
              size_t len,
              uint32_t prot,
              uint32_t flags)
{
    if (pa == NULL || len == 0) {
        return 0;
    }

    if (flags & MAP_FIXED) {
        return aspace_map_at(pa, addr, len, prot);
    }

    return aspace_map_anon(pa, addr, len, prot);
}

bool munmap(proc_aspace_t *pa, uint64_t addr, size_t len)
{
    return aspace_unmap(pa, addr, len);
}

bool mprotect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
{
    return aspace_protect(pa, addr, len, prot);
}