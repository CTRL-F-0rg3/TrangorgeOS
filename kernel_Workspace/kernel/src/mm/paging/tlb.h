#ifndef MM_PAGING_TLB_H
#define MM_PAGING_TLB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define TLB_BATCH_MAX 16

typedef struct tlb_batch {
    uint64_t pages[TLB_BATCH_MAX];
    size_t count;
    bool full_flush;
} tlb_batch_t;

void tlb_batch_begin(tlb_batch_t *batch);
void tlb_batch_add(tlb_batch_t *batch, uint64_t virt);
void tlb_batch_full(tlb_batch_t *batch);
void tlb_batch_commit(tlb_batch_t *batch);

#endif