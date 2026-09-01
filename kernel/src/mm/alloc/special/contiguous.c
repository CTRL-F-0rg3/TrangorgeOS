#include "contiguous.h"
#include "../physical/pmm.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"


static bool contig_bytes_to_frames(size_t bytes, size_t *out_frames)
{
	return size_bytes_to_pages_checked(bytes, ARCH_PAGE_SIZE, out_frames);
}

bool contig_alloc(size_t bytes,
	              size_t align,
	              uint64_t *out_phys,
	              void **out_virt)
{
	if (bytes == 0) {
	    return false;
	}


	if (align != 0 && !size_is_pow2(align)) {
	    return false;
	}

	size_t frames;

	if (!contig_bytes_to_frames(bytes, &frames)) {
	    return false;
	}

	size_t align_frames = 1;

	if (align > ARCH_PAGE_SIZE) {

	    align_frames = align / ARCH_PAGE_SIZE;
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

	size_t frames;

	if (!contig_bytes_to_frames(bytes, &frames)) {

	    return;
	}

	pmm_free_frames(phys, frames);
}