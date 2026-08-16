#include "paging.h"
#include "memory.h"
#include "../../alloc/physical/pmm.h"

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

/*
 * PTE flags that paging_set_flags() may change on an existing mapping
 * (PTE_PRESENT and PTE_PAGE_SIZE are preserved separately).
 */
#define PTE_FLAGS_MASK \
    (PTE_WRITABLE | PTE_USER | PTE_WRITE_THROUGH | PTE_CACHE_DISABLE | \
     PTE_ACCESSED | PTE_DIRTY | PTE_GLOBAL | PTE_NX)

/* IA32_EFER (MSR 0xC0000080) — the NXE bit enables Execute-Disable. */
#define IA32_EFER_MSR 0xC0000080U
#define EFER_NXE_BIT  (1U << 11)

static bool nx_enabled = false;

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

static inline uint64_t read_msr(uint32_t msr)
{
    uint32_t low;
    uint32_t high;

    __asm__ volatile("rdmsr" : "=a"(low), "=d"(high) : "c"(msr));

    return ((uint64_t)high << 32) | low;
}

static inline void write_msr(uint32_t msr, uint64_t value)
{
    uint32_t low = (uint32_t)value;
    uint32_t high = (uint32_t)(value >> 32);

    __asm__ volatile("wrmsr" :: "a"(low), "d"(high), "c"(msr));
}

static void cpuid_leaf(uint32_t leaf,
                       uint32_t subleaf,
                       uint32_t *eax,
                       uint32_t *ebx,
                       uint32_t *ecx,
                       uint32_t *edx)
{
    __asm__ volatile("cpuid"
                     : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                     : "a"(leaf), "c"(subleaf));
}

static bool cpuid_max_standard_leaf_at_least(uint32_t leaf)
{
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;

    cpuid_leaf(0, 0, &eax, &ebx, &ecx, &edx);

    return eax >= leaf;
}

static bool cpuid_leaf7_ebx_bit(uint32_t bit)
{
    if (!cpuid_max_standard_leaf_at_least(7)) {
        return false;
    }

    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;

    cpuid_leaf(7, 0, &eax, &ebx, &ecx, &edx);

    return (ebx & (1U << bit)) != 0;
}

static bool cpuid_leaf7_ecx_bit(uint32_t bit)
{
    if (!cpuid_max_standard_leaf_at_least(7)) {
        return false;
    }

    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;

    cpuid_leaf(7, 0, &eax, &ebx, &ecx, &edx);

    return (ecx & (1U << bit)) != 0;
}

static bool cpu_supports_nx(void)
{
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;

    cpuid_leaf(0x80000001U, 0, &eax, &ebx, &ecx, &edx);

    return (edx & (1U << 20)) != 0;
}

void paging_enable_nx(void)
{
    if (nx_enabled) {
        return;
    }

    if (!cpu_supports_nx()) {
        return;
    }

    write_msr(IA32_EFER_MSR, read_msr(IA32_EFER_MSR) | EFER_NXE_BIT);

    nx_enabled = true;
}

bool paging_nx_enabled(void)
{
    return nx_enabled;
}

static inline uint64_t read_cr0(void)
{
    uint64_t cr0;

    __asm__ volatile("mov %%cr0, %0" : "=r"(cr0));

    return cr0;
}

static inline void write_cr0(uint64_t cr0)
{
    __asm__ volatile("mov %0, %%cr0" :: "r"(cr0) : "memory");
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

#define CR0_WRITE_PROTECT (1ULL << 16)

void paging_enable_write_protect(void)
{
    write_cr0(read_cr0() | CR0_WRITE_PROTECT);
}

void paging_disable_write_protect(void)
{
    write_cr0(read_cr0() & ~CR0_WRITE_PROTECT);
}

bool paging_write_protect_enabled(void)
{
    return (read_cr0() & CR0_WRITE_PROTECT) != 0;
}

#define CR4_SMEP  (1ULL << 20)
#define CR4_SMAP  (1ULL << 21)
#define CR4_PCIDE (1ULL << 17)
#define CR4_LA57  (1ULL << 12)

void paging_enable_smep(void)
{
    if (cpuid_leaf7_ebx_bit(7)) {
        write_cr4(read_cr4() | CR4_SMEP);
    }
}

void paging_enable_smap(void)
{
    if (cpuid_leaf7_ebx_bit(20)) {
        write_cr4(read_cr4() | CR4_SMAP);
    }
}

bool paging_smep_enabled(void)
{
    return (read_cr4() & CR4_SMEP) != 0;
}

bool paging_smap_enabled(void)
{
    return (read_cr4() & CR4_SMAP) != 0;
}

bool paging_pcid_supported(void)
{
    return cpuid_leaf7_ebx_bit(17);
}

void paging_enable_pcid(void)
{
    if (paging_pcid_supported()) {
        write_cr4(read_cr4() | CR4_PCIDE);
    }
}

bool paging_pcid_enabled(void)
{
    return (read_cr4() & CR4_PCIDE) != 0;
}

bool paging_la57_supported(void)
{
    return cpuid_leaf7_ecx_bit(16);
}

void paging_assert_4level_paging(void)
{
    if (read_cr4() & CR4_LA57) {
        paging_panic("5-level paging (LA57) enabled — not supported");
    }
}

/*
 * Raw wrapper for INVPCID. Requires CR4.PCIDE to be enabled.
 *   type: 0 = individual, 1 = single-context (global),
 *         2 = all-contexts (incl. global), 3 = single-context (non-global).
 */
void paging_invpcid(uint64_t type, uint64_t pcid, uint64_t addr)
{
    struct invpcid_desc {
        uint64_t pcid;
        uint64_t addr;
    } desc = { pcid & 0xFFFULL, addr };

    __asm__ volatile("invpcid %0, %1" : : "m"(desc), "r"(type) : "memory");
}

static uint64_t alloc_zeroed_table_page(void)
{
    uint64_t phys = 0;

    if (pmm_ready()) {
        if (!pmm_alloc_zero_frame(&phys)) {
            paging_panic("cannot allocate page table page");
        }
    } else {
        if (!arch_memory_boot_alloc(PAGING_PAGE_SIZE,
                                    PAGING_PAGE_SIZE,
                                    &phys)) {
            paging_panic("cannot allocate page table page");
        }

        zero_page_phys(phys);
    }

    return phys;
}

static void free_table_page(uint64_t phys)
{
    /*
     * Before pmm_init() frames come from the boot allocator and cannot be
     * freed — we allow the leak (early boot only).
     */
    if (pmm_ready()) {
        pmm_free_frame(phys);
    }
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
     * Intermediate tables must be kernel-accessible. If the target mapping
     * is user-space, set USER as well.
     */
    uint64_t table_flags = PTE_PRESENT | PTE_WRITABLE;

    if (flags & PTE_USER) {
        table_flags |= PTE_USER;
    }

    table[index] = (table_phys & PAGING_ADDR_MASK) | table_flags;

    return table_phys;
}

typedef enum page_level {
    PAGE_LEVEL_4K = 0,
    PAGE_LEVEL_2M = 1,
    PAGE_LEVEL_1G = 2,
} page_level_t;

/*
 * Walks the PML4 -> PDPT -> PD -> PT tree and returns a pointer to the
 * leaf entry for `virt`, without creating any tables.
 *
 * Returns a pointer to a PT entry (4 KiB), PD entry (2 MiB) or PDPT entry
 * (1 GiB), or NULL when the address is not mapped. `out_level` receives the
 * level at which the leaf was found.
 */
static uint64_t *walk_leaf(uint64_t *pml4,
                           uint64_t virt,
                           page_level_t *out_level)
{
    uint64_t pml4e = pml4[pml4_index(virt)];

    if (!(pml4e & PTE_PRESENT)) {
        return NULL;
    }

    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pml4e & PAGING_ADDR_MASK);
    uint64_t pdpte = pdpt[pdpt_index(virt)];

    if (!(pdpte & PTE_PRESENT)) {
        return NULL;
    }

    if (pdpte & PTE_PAGE_SIZE) {
        if (out_level != NULL) {
            *out_level = PAGE_LEVEL_1G;
        }

        return &pdpt[pdpt_index(virt)];
    }

    uint64_t *pd = (uint64_t *)phys_to_ptr(pdpte & PAGING_ADDR_MASK);
    uint64_t pde = pd[pd_index(virt)];

    if (!(pde & PTE_PRESENT)) {
        return NULL;
    }

    if (pde & PTE_PAGE_SIZE) {
        if (out_level != NULL) {
            *out_level = PAGE_LEVEL_2M;
        }

        return &pd[pd_index(virt)];
    }

    uint64_t *pt = (uint64_t *)phys_to_ptr(pde & PAGING_ADDR_MASK);
    uint64_t *pte = &pt[pt_index(virt)];

    if (!(*pte & PTE_PRESENT)) {
        return NULL;
    }

    if (out_level != NULL) {
        *out_level = PAGE_LEVEL_4K;
    }

    return pte;
}

/*
 * Walks to the PT entry (4 KiB page) for `virt`, creating any missing
 * intermediate tables along the way. Returns a pointer to the PT entry.
 */
static uint64_t *walk_to_pt(uint64_t *pml4, uint64_t virt, uint64_t flags)
{
    uint64_t pdpt_phys = ensure_table_entry(pml4, pml4_index(virt), flags);
    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pdpt_phys);

    uint64_t pd_phys = ensure_table_entry(pdpt, pdpt_index(virt), flags);
    uint64_t *pd = (uint64_t *)phys_to_ptr(pd_phys);

    uint64_t pt_phys = ensure_table_entry(pd, pd_index(virt), flags);
    uint64_t *pt = (uint64_t *)phys_to_ptr(pt_phys);

    return &pt[pt_index(virt)];
}

bool paging_map_page_in(uint64_t pml4_phys,
                        uint64_t virt,
                        uint64_t phys,
                        uint64_t flags)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    if ((virt & PAGING_PAGE_MASK) != 0 ||
        (phys & PAGING_PAGE_MASK) != 0) {
        return false;
    }

    flags &= ~PTE_PAGE_SIZE;

    /*
     * PTE_NX only makes sense once EFER.NXE is enabled. Before that, setting
     * it causes a page fault on every access to the page.
     */
    if (!nx_enabled) {
        flags &= ~PTE_NX;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    uint64_t *pte = walk_to_pt(pml4, virt, flags);

    *pte = (phys & PAGING_ADDR_MASK) | flags | PTE_PRESENT;

    paging_flush_page(virt);

    return true;
}

bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags)
{
    return paging_map_page_in(paging_read_cr3(), virt, phys, flags);
}

static bool table_is_empty(const uint64_t *table)
{
    for (size_t i = 0; i < 512; i++) {
        if (table[i] & PTE_PRESENT) {
            return false;
        }
    }

    return true;
}

/*
 * Recursively frees an intermediate table and all of its child tables.
 * `level`: 2 = PDPT, 1 = PD, 0 = PT. Large pages (1 GiB / 2 MiB) have no
 * child tables, so they are skipped — we only free the tables themselves.
 */
static void free_page_table(uint64_t table_phys, unsigned level)
{
    uint64_t *table = (uint64_t *)phys_to_ptr(table_phys);

    for (size_t i = 0; i < 512; i++) {
        uint64_t entry = table[i];

        if (!(entry & PTE_PRESENT)) {
            continue;
        }

        if (entry & PTE_PAGE_SIZE) {
            continue;
        }

        if (level > 0) {
            free_page_table(entry & PAGING_ADDR_MASK, level - 1);
        }
    }

    free_table_page(table_phys);
}

uint64_t paging_create_pml4(void)
{
    if (!boot_phys_offset_valid) {
        return 0;
    }

    uint64_t pml4_phys = 0;

    if (pmm_ready()) {
        if (!pmm_alloc_zero_frame(&pml4_phys)) {
            return 0;
        }
    } else {
        if (!arch_memory_boot_alloc(PAGING_PAGE_SIZE,
                                    PAGING_PAGE_SIZE,
                                    &pml4_phys)) {
            return 0;
        }

        zero_page_phys(pml4_phys);
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    uint64_t *current = (uint64_t *)phys_to_ptr(paging_read_cr3());

    /*
     * Copy the higher half (kernel) from the current PML4, so the new
     * address space sees the direct map and the whole kernel.
     */
    for (size_t i = PAGING_KERNEL_PML4_START; i < 512; i++) {
        pml4[i] = current[i];
    }

    return pml4_phys;
}

void paging_destroy_pml4(uint64_t pml4_phys)
{
    if (!boot_phys_offset_valid) {
        return;
    }

    if (pml4_phys == paging_read_cr3()) {
        paging_panic("cannot destroy active PML4");
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);

    /*
     * We only free the lower half (user-space). The upper half is shared
     * with the kernel and other address spaces.
     */
    for (size_t i = 0; i < PAGING_KERNEL_PML4_START; i++) {
        uint64_t entry = pml4[i];

        if ((entry & PTE_PRESENT) && !(entry & PTE_PAGE_SIZE)) {
            free_page_table(entry & PAGING_ADDR_MASK, 2);
        }
    }

    free_table_page(pml4_phys);
}

void paging_switch_pml4(uint64_t pml4_phys)
{
    __asm__ volatile("mov %0, %%cr3" :: "r"(pml4_phys) : "memory");
}

void paging_write_cr3(uint64_t pml4_phys)
{
    __asm__ volatile("mov %0, %%cr3" :: "r"(pml4_phys) : "memory");
}

/*
 * After removing a 4 KiB page, checks whether the intermediate tables
 * (PT/PD/PDPT) have become empty and frees them. Only works in the lower
 * half — kernel tables are shared and must not be freed.
 */
static void free_empty_tables(uint64_t *pml4, uint64_t virt)
{
    size_t pml4i = pml4_index(virt);

    if (pml4i >= PAGING_KERNEL_PML4_START) {
        return;
    }

    uint64_t pml4e = pml4[pml4i];

    if (!(pml4e & PTE_PRESENT)) {
        return;
    }

    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pml4e & PAGING_ADDR_MASK);
    size_t pdpti = pdpt_index(virt);
    uint64_t pdpte = pdpt[pdpti];

    if (!(pdpte & PTE_PRESENT) || (pdpte & PTE_PAGE_SIZE)) {
        return;
    }

    uint64_t *pd = (uint64_t *)phys_to_ptr(pdpte & PAGING_ADDR_MASK);
    size_t pdi = pd_index(virt);
    uint64_t pde = pd[pdi];

    if (!(pde & PTE_PRESENT) || (pde & PTE_PAGE_SIZE)) {
        return;
    }

    uint64_t *pt = (uint64_t *)phys_to_ptr(pde & PAGING_ADDR_MASK);

    if (!table_is_empty(pt)) {
        return;
    }

    free_table_page(pde & PAGING_ADDR_MASK);
    pd[pdi] = 0;

    if (!table_is_empty(pd)) {
        return;
    }

    free_table_page(pdpte & PAGING_ADDR_MASK);
    pdpt[pdpti] = 0;

    if (!table_is_empty(pdpt)) {
        return;
    }

    free_table_page(pml4e & PAGING_ADDR_MASK);
    pml4[pml4i] = 0;
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

    flags &= ~PTE_PAGE_SIZE;

    if (!nx_enabled) {
        flags &= ~PTE_NX;
    }

    uint64_t pml4_phys = paging_read_cr3();
    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);

    uint64_t pdpt_phys = ensure_table_entry(pml4, pml4_index(virt), flags);
    uint64_t *pdpt = (uint64_t *)phys_to_ptr(pdpt_phys);

    uint64_t pd_phys = ensure_table_entry(pdpt, pdpt_index(virt), flags);
    uint64_t *pd = (uint64_t *)phys_to_ptr(pd_phys);

    size_t idx = pd_index(virt);
    uint64_t old = pd[idx];

    /*
     * If a 4 KiB table already exists here, do not overwrite it with a
     * 2 MiB page.
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

bool paging_map_mmio(uint64_t virt, uint64_t phys, uint64_t len)
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

    uint64_t end = virt + len;

    while (virt < end) {
        if (!paging_map_page(virt,
                             phys,
                             PAGING_KERNEL_RW |
                             PTE_CACHE_DISABLE |
                             PTE_WRITE_THROUGH)) {
            return false;
        }

        virt += PAGING_PAGE_SIZE;
        phys += PAGING_PAGE_SIZE;
    }

    return true;
}

bool paging_unmap_page_in(uint64_t pml4_phys, uint64_t virt)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    if ((virt & PAGING_PAGE_MASK) != 0) {
        return false;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    page_level_t level = PAGE_LEVEL_4K;
    uint64_t *leaf = walk_leaf(pml4, virt, &level);

    if (leaf == NULL) {
        return false;
    }

    *leaf = 0;

    paging_flush_page(virt);

    if (level == PAGE_LEVEL_4K) {
        free_empty_tables(pml4, virt);
    }

    return true;
}

bool paging_unmap_page(uint64_t virt)
{
    return paging_unmap_page_in(paging_read_cr3(), virt);
}

/*
 * Translates a virtual address to a physical one. Returns 0 when the address
 * is not mapped (physical address 0 is then indistinguishable — use
 * paging_is_mapped() for an unambiguous check).
 */
uint64_t paging_translate_in(uint64_t pml4_phys, uint64_t virt)
{
    if (!boot_phys_offset_valid) {
        return 0;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    page_level_t level = PAGE_LEVEL_4K;
    uint64_t *leaf = walk_leaf(pml4, virt, &level);

    if (leaf == NULL) {
        return 0;
    }

    uint64_t entry = *leaf;

    switch (level) {
    case PAGE_LEVEL_1G:
        return (entry & PAGING_ADDR_MASK) | (virt & PAGING_1G_PAGE_MASK);
    case PAGE_LEVEL_2M:
        return (entry & PAGING_ADDR_MASK) | (virt & PAGING_2M_PAGE_MASK);
    default:
        return (entry & PAGING_ADDR_MASK) | (virt & PAGING_PAGE_MASK);
    }
}

uint64_t paging_translate(uint64_t virt)
{
    return paging_translate_in(paging_read_cr3(), virt);
}

bool paging_is_mapped_in(uint64_t pml4_phys, uint64_t virt)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);

    return walk_leaf(pml4, virt, NULL) != NULL;
}

bool paging_is_mapped(uint64_t virt)
{
    return paging_is_mapped_in(paging_read_cr3(), virt);
}

bool paging_set_flags_in(uint64_t pml4_phys, uint64_t virt, uint64_t flags)
{
    if (!boot_phys_offset_valid) {
        return false;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    uint64_t *leaf = walk_leaf(pml4, virt, NULL);

    if (leaf == NULL) {
        return false;
    }

    if (!nx_enabled) {
        flags &= ~PTE_NX;
    }

    uint64_t old = *leaf;

    *leaf = (old & PAGING_ADDR_MASK) |
            PTE_PRESENT |
            (old & PTE_PAGE_SIZE) |
            (flags & PTE_FLAGS_MASK);

    paging_flush_page(virt);

    return true;
}

bool paging_set_flags(uint64_t virt, uint64_t flags)
{
    return paging_set_flags_in(paging_read_cr3(), virt, flags);
}

bool paging_get_flags_in(uint64_t pml4_phys, uint64_t virt, uint64_t *out_flags)
{
    if (out_flags == NULL) {
        return false;
    }

    if (!boot_phys_offset_valid) {
        return false;
    }

    uint64_t *pml4 = (uint64_t *)phys_to_ptr(pml4_phys);
    uint64_t *leaf = walk_leaf(pml4, virt, NULL);

    if (leaf == NULL) {
        return false;
    }

    *out_flags = *leaf;

    return true;
}

bool paging_get_flags(uint64_t virt, uint64_t *out_flags)
{
    return paging_get_flags_in(paging_read_cr3(), virt, out_flags);
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
     * If the bootloader already maps physical memory exactly at
     * ARCH_DIRECT_MAP_BASE, there is nothing to do.
     */
    if (boot_phys_offset == ARCH_DIRECT_MAP_BASE) {
        return;
    }

    const arch_mem_info_t *info = arch_memory_get();

    if (info == NULL) {
        paging_panic("no arch memory info");
    }

    uint64_t phys_limit = arch_page_align_up(info->max_usable_address);

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
    paging_assert_4level_paging();

    paging_set_boot_phys_offset(phys_offset);
    paging_init_direct_map();
}
