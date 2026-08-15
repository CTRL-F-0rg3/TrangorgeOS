#include "mmap.h"
#include "../protection/permissions.h"

uint64_t mmap(proc_aspace_t *pa,
              uint64_t addr,
              size_t len,
              uint32_t prot,
              uint32_t flags)
{
    if (pa == NULL || len == 0) {
        return 0;
    }

    prot = perm_sanitize(prot);

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
    vma_t *v = aspace_vma_find(pa, addr);

    if (v == NULL) {
        return false;
    }

    if (!perm_mprotect_allowed(v->prot, prot)) {
        return false;
    }

    return aspace_protect(pa, addr, len, perm_sanitize(prot));
}