#include "contiguous.h"
#include "../physical/pmm.h"
#include "../../arch/x86_64/memory.h"

static size_t contig_bytes_to_frames(size_t bytes)
{
    return (bytes + ARCH_PAGE_SIZE - 1) / ARCH_PAGE_SIZE;
}

bool contig_alloc(size_t bytes,
                  size_t align,
                  uint64_t *out_phys,
                  void **out_virt)
{
    if (bytes == 0) {
        return false;
    }

    size_t frames = contig_bytes_to_frames(bytes);

    size_t align_frames = 1;

    if (align > ARCH_PAGE_SIZE) {
        uint64_t a = arch_page_align_up((uint64_t)align);
        align_frames = (size_t)(a / ARCH_PAGE_SIZE);
    }

    uint64_t phys = 0;

    if (!pmm_alloc_frames_aligned(frames, align_frames, &phys)) {
        return false;
    }

    if (out_phys != NULL) {
        *out_phys = phys;
    }

    if (out_virt != NULL) {
        *out_virt = arch_phys_to_virt(phys);
    }

    return true;
}

void contig_free(uint64_t phys, size_t bytes)
{
    if (bytes == 0) {
        return;
    }

    pmm_free_frames(phys, contig_bytes_to_frames(bytes));
}