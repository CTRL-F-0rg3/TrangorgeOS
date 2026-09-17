#ifndef MM_ALLOC_VIRTUAL_PAGE_H
#define MM_ALLOC_VIRTUAL_PAGE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define PAGE_TYPE_FREE   0u
#define PAGE_TYPE_KERNEL 1u
#define PAGE_TYPE_USER   2u
#define PAGE_TYPE_SLAB   3u
#define PAGE_TYPE_TABLE  4u
#define PAGE_TYPE_DEVICE 5u

typedef struct page {
    uint32_t refcount;
    uint32_t type;
    uint32_t flags;
    uint32_t reserved;
} page_t;

bool page_init(void);
bool page_ready(void);

page_t *page_alloc(uint32_t type);
page_t *page_alloc_zero(uint32_t type);

bool page_attach(uint64_t phys, uint32_t type);

bool page_get(page_t *page);
bool page_put(page_t *page);

uint64_t page_phys(const page_t *page);
void *page_kva(const page_t *page);
size_t page_pfn(const page_t *page);

page_t *page_from_phys(uint64_t phys);
page_t *page_from_pfn(size_t pfn);
page_t *page_from_kva(const void *va);

uint32_t page_refcount(const page_t *page);
uint32_t page_type(const page_t *page);
void page_set_type(page_t *page, uint32_t type);

size_t page_stat_total(void);
size_t page_stat_referenced(void);

void page_dump(void);

#endif