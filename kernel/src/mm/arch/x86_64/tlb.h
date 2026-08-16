#ifndef ARCH_X86_64_TLB_H
#define ARCH_X86_64_TLB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define TLB_PAGE_SHIFT 12
#define TLB_PAGE_SIZE  (1ULL << TLB_PAGE_SHIFT)
#define TLB_PAGE_MASK  (TLB_PAGE_SIZE - 1ULL)

#define TLB_PCID_KERNEL 0

bool tlb_init(void);
bool tlb_ready(void);
bool tlb_has_pcid(void);
bool tlb_has_invpcid(void);

void tlb_flush_all(void);
void tlb_flush_all_including_global(void);

void tlb_flush_page(const void *addr);
void tlb_flush_page_addr(uint64_t addr);

void tlb_flush_range(const void *addr, size_t pages);
void tlb_flush_range_addr(uint64_t addr, size_t pages);

void tlb_flush_pcid(uint16_t pcid);
void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr);

void tlb_wbinvd(void);
void tlb_clflush(uint64_t addr);

#endif