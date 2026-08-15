#include "paging.h"
#include "memory.h"

extern void kprintf(const char *fmt, ...);

static void paging_panic(const char *msg) __attribute__((noreturn));
static void paging_panic(const char *msg)
{
    kprintf("arch/x86_64/paging.c panic: %s\n", msg);

    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static uint64_t boot_phys_offset = 0;
static bool boot_phys_offset_valid = false;

static void *phys_to_ptr(uint64_t phys)
{
    if (!boot_phys_offset_valid) {
        paging_panic("boot physical offset not set");
    }

    return (void *)(uintptr_t)(phys + boot_phys_offset);
}

static void zero_page_phys(uint64_t phys)
{
    uint64_t *p = (uint64_t *)phys_to_ptr(phys);

    for (size_t i = 0; i < 512; i++) {
        p[i] = 0;
    }

    __asm__ volatile("" ::: "memory");
}

static inline size_t pml4_index(uint64_t virt)
{
    return (size_t)((virt >> 39) & 0x1FF);
}

static inline size_t pdpt_index(uint64_t virt)
{
    return (size_t)((virt >> 30) & 0x1FF);
}

static inline size_t pd_index(uint64_t virt)
{
    return (size_t)((virt >> 21) & 0x1FF);
}

static inline size_t pt_index(uint64_t virt)
{
    return (size_t)((virt >> 12) & 0x1FF);
}

uint64_t paging_read_cr3(void)
{
    uint64_t cr3;

    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));

    return cr3 & PAGING_ADDR_MASK;
}

void paging_flush_tlb_all(void)
{
    uint64_t cr3;

    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));
    __asm__ volatile("mov %0, %%cr3" :: "r"(cr3) : "memory");
}

void paging_flush_page(uint64_t addr)
{
    __asm__ volatile("invlpg (%0)" :: "r"(addr) : "memory");
}

static uint64_t alloc_zeroed_table_page(void)
{
    uint64_t phys = 0;

    if (!arch_memory_boot_alloc(PAGING_PAGE_SIZE,
                                PAGING_PAGE_SIZE,
                                &phys)) {
        paging_panic("cannot allocate page table page");
    }

    zero_page_phys(phys);

    return phys;
}

static uint64_t ensure_table_entry(uint64_t *table,
                                   size_t index,
                                   uint64_t flags)
{
    uint64_t entry = table[index];

    if (entry & PTE_PRESENT) {
        if (entry & PTE_PAGE_SIZE) {
            paging_panic("expected page table, found large page");
        }

        return entry & PAGING_ADDR_MASK;
    }

    uint64_t table_phys = alloc_zeroed_table_page();

    /*
     * Tablice pośrednie muszą być dostępne dla kernela.
     * Jeśli docelowe mapowanie ma być user-space, ustawiamy też USER.
     */
    uint64_t table_flags = PTE_PRESENT | PTE_WRITABLE;

    if (flags & PTE_USER) {
        table_flags |= PTE_USER;
    }

    table[index] = (table_phys & PAGING_ADDR_MASK) | table_flags;

    return table_phys;
}

bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    if ((virt & PAGING_PAGE_MASK) != 0 ||
        (phys & PAGING_PAGE_MASK) != 0) {
        return false;
    }

    /*
     * Tymczasowo nie używamy NX, dopóki EFER.NXE nie jest włączony.
     */
    flags &= ~PTE_NX;
    flags &= ~PTE_PAGE_SIZE;

    uint64_t pml4_phys = paging_read_cr3();
    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);

    uint64_t pdpt_phys = ensure_table_entry(pml4, pml4_index(virt), flags);
    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pdpt_phys);

    uint64_t pd_phys = ensure_table_entry(pdpt, pdpt_index(virt), flags);
    uint64_t *pd = (uint64_t *)phys_to_ptr(pd_phys);

    uint64_t pt_phys = ensure_table_entry(pd, pd_index(virt), flags);
    uint64_t *pt = (uint64_t *)phys_to_ptr(pt_phys);

    pt[pt_index(virt)] =
        (phys & PAGING_ADDR_MASK) |
        flags |
        PTE_PRESENT;

    paging_flush_page(virt);

    return true;
}


static bool map_page_2m(uint64_t virt, uint64_t phys, uint64_t flags)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    if ((virt & PAGING_2M_PAGE_MASK) != 0 ||
        (phys & PAGING_2M_PAGE_MASK) != 0) {
        return false;
    }

    flags &= ~PTE_NX;
    flags &= ~PTE_PAGE_SIZE;

    uint64_t pml4_phys = paging_read_cr3();
    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);

    uint64_t pdpt_phys = ensure_table_entry(pml4, pml4_index(virt), flags);
    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pdpt_phys);

    uint64_t pd_phys = ensure_table_entry(pdpt, pdpt_index(virt), flags);
    uint64_t *pd = (uint64_t *)phys_to_ptr(pd_phys);

    size_t idx = pd_index(virt);
    uint64_t old = pd[idx];

    /*
     * Jeśli istnieje już tablica 4 KiB w tym miejscu,
     * nie nadpisujemy jej 2 MiB stroną.
     */
    if ((old & PTE_PRESENT) && !(old & PTE_PAGE_SIZE)) {
        paging_panic("cannot map 2M page over existing 4K table");
    }

    pd[idx] =
        (phys & PAGING_ADDR_MASK) |
        flags |
        PTE_PRESENT |
        PTE_PAGE_SIZE;

    return true;
}

bool paging_map_range(uint64_t virt, uint64_t phys, uint64_t len, uint64_t flags)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    if (len == 0) {
        return true;
    }

    if ((virt & PAGING_PAGE_MASK) != 0 ||
        (phys & PAGING_PAGE_MASK) != 0 ||
        (len & PAGING_PAGE_MASK) != 0) {
        return false;
    }

    if (len > UINT64_MAX - virt || len > UINT64_MAX - phys) {
        return false;
    }

    uint64_t virt_end = virt + len;
    uint64_t phys_end = phys + len;

    while (virt < virt_end) {
        uint64_t remaining = virt_end - virt;

        bool can_use_2m =
            (remaining >= PAGING_2M_PAGE_SIZE) &&
            ((virt & PAGING_2M_PAGE_MASK) == 0) &&
            ((phys & PAGING_2M_PAGE_MASK) == 0);

        if (can_use_2m) {
            if (!map_page_2m(virt, phys, flags)) {
                return false;
            }

            virt += PAGING_2M_PAGE_SIZE;
            phys += PAGING_2M_PAGE_SIZE;
        } else {
            if (!paging_map_page(virt, phys, flags)) {
                return false;
            }

            virt += PAGING_PAGE_SIZE;
            phys += PAGING_PAGE_SIZE;
        }
    }

    paging_flush_tlb_all();

    return true;
}

void paging_set_boot_phys_offset(uint64_t phys_offset)
{
    if ((phys_offset & PAGING_PAGE_MASK) != 0) {
        paging_panic("boot physical offset must be page aligned");
    }

    boot_phys_offset = phys_offset;
    boot_phys_offset_valid = true;
}

bool paging_boot_phys_offset_valid(void)
{
    return boot_phys_offset_valid;
}

void paging_init_direct_map(void)
{
    if (!arch_memory_ready()) {
        paging_panic("arch memory not initialized");
    }

    if (!boot_phys_offset_valid) {
        paging_panic("boot physical offset not set");
    }

    /*
     * Jeśli bootloader już mapuje fizyczną pamięć dokładnie pod
     * ARCH_DIRECT_MAP_BASE, nie musimy nic robić.
     */
    if (boot_phys_offset == ARCH_DIRECT_MAP_BASE) {
        return;
    }

    const arch_mem_info_t *info = arch_memory_get();

    if (info == NULL) {
        paging_panic("no arch memory info");
    }

    uint64_t phys_limit = arch_page_align_up(info->max_address);

    if (phys_limit == 0 || phys_limit == UINT64_MAX) {
        paging_panic("invalid physical memory limit");
    }

    if (!paging_map_range(ARCH_DIRECT_MAP_BASE,
                          0,
                          phys_limit,
                          PAGING_KERNEL_RW)) {
        paging_panic("failed to create direct map");
    }

    paging_flush_tlb_all();
}

void paging_init(uint64_t phys_offset)
{
    paging_set_boot_phys_offset(phys_offset);
    paging_init_direct_map();
}
