#include "kstd_mem.h"

extern void *kmalloc(unsigned long size);
extern void kfree(void *ptr);
extern bool vmm_map_device(uint64_t phys, unsigned long len, uint64_t *virt);
extern uint64_t kvirt_to_phys(void *ptr);

void *tr_alloc(uint32_t bytes)
{
    return kmalloc(bytes);
}

void tr_free(void *ptr, uint32_t bytes)
{
    (void)bytes;
    kfree(ptr);
}

tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va)
{
    uint64_t v = 0;

    if (vmm_map_device(phys, len, &v)) {
        *out_va = (void *)v;
        return TR_OK;
    }

    return TR_ERR_IO;
}