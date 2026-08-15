#include "tlb.h"
#include "../arch/x86_64/tlb.h"

void tlb_batch_begin(tlb_batch_t *batch)
{
    batch->count = 0;
    batch->full_flush = false;
}

void tlb_batch_full(tlb_batch_t *batch)
{
    batch->full_flush = true;
}

void tlb_batch_add(tlb_batch_t *batch, uint64_t virt)
{
    if (batch->full_flush) {
        return;
    }

    if (batch->count >= TLB_BATCH_MAX) {
        batch->full_flush = true;
        batch->count = 0;
        return;
    }

    batch->pages[batch->count++] = virt;
}

void tlb_batch_commit(tlb_batch_t *batch)
{
    if (batch->full_flush) {
        tlb_flush_all();
        batch->count = 0;
        batch->full_flush = false;
        return;
    }

    for (size_t i = 0; i < batch->count; i++) {
        tlb_flush_page_addr(batch->pages[i]);
    }

    batch->count = 0;
}