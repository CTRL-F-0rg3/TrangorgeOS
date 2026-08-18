#ifndef ARCH_AARCH64_TLB_H
#define ARCH_AARCH64_TLB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool tlb_init(void);
bool tlb_ready(void);

void tlb_flush_all(void);
void tlb_flush_all_including_global(void);

void tlb_flush_page(const void *addr);
void tlb_flush_page_addr(uint64_t addr);

void tlb_flush_range(const void *addr, size_t pages);
void tlb_flush_range_addr(uint64_t addr, size_t pages);

#endif