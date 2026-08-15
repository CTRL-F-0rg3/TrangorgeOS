#include "tlb.h"

#define TLB_CR4_PGE (1ULL << 7)
#define TLB_RANGE_FLUSH_THRESHOLD 32

static inline uint64_t read_cr3(void)
{
    uint64_t cr3;

    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));

    return cr3;
}

static inline void write_cr3(uint64_t cr3)
{
    __asm__ volatile("mov %0, %%cr3" :: "r"(cr3) : "memory");
}

static inline uint64_t read_cr4(void)
{
    uint64_t cr4;

    __asm__ volatile("mov %%cr4, %0" : "=r"(cr4));

    return cr4;
}

static inline void write_cr4(uint64_t cr4)
{
    __asm__ volatile("mov %0, %%cr4" :: "r"(cr4) : "memory");
}

static inline void invlpg_addr(uint64_t addr)
{
    __asm__ volatile("invlpg (%0)" :: "r"(addr) : "memory");
}

void tlb_flush_page_addr(uint64_t addr)
{
    addr &= ~TLB_PAGE_MASK;

    invlpg_addr(addr);

    __asm__ volatile("" ::: "memory");
}

void tlb_flush_page(const void *addr)
{
    if (addr == NULL) {
        tlb_flush_all();
        return;
    }

    tlb_flush_page_addr((uint64_t)(uintptr_t)addr);
}

void tlb_flush_all(void)
{
    uint64_t cr3 = read_cr3();

    /*
     * Przeładowanie CR3 wyrzuca większość TLB.
     *
     * Uwaga: jeśli CR4.PGE jest włączone, global pages mogą pozostać.
     */
    write_cr3(cr3);
}

void tlb_flush_all_including_global(void)
{
    uint64_t cr4 = read_cr4();

    if (cr4 & TLB_CR4_PGE) {
        /*
         * Wyłączenie i ponowne włączenie PGE wyrzuca również
         * globalne wpisy TLB.
         */
        write_cr4(cr4 & ~TLB_CR4_PGE);
        write_cr4(cr4);
    } else {
        tlb_flush_all();
    }
}

void tlb_flush_range_addr(uint64_t addr, size_t pages)
{
    if (pages == 0) {
        return;
    }

    /*
     * Dla dużych zakresów full flush jest zwykle szybszy.
     */
    if (pages > TLB_RANGE_FLUSH_THRESHOLD) {
        tlb_flush_all();
        return;
    }

    addr &= ~TLB_PAGE_MASK;

    /*
     * Bezpieczeństwo przeciw overflowowi.
     */
    if (pages > (UINT64_MAX - addr) / TLB_PAGE_SIZE) {
        tlb_flush_all();
        return;
    }

    for (size_t i = 0; i < pages; i++) {
        invlpg_addr(addr);
        addr += TLB_PAGE_SIZE;
    }

    __asm__ volatile("" ::: "memory");
}

void tlb_flush_range(const void *addr, size_t pages)
{
    if (addr == NULL) {
        tlb_flush_all();
        return;
    }

    tlb_flush_range_addr((uint64_t)(uintptr_t)addr, pages);
}