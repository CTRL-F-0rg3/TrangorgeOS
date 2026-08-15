#include "vmm.h"
#include "../physical/pmm.h"
#include "../physical/bitmap.h"
#include "../../arch/x86_64/paging.h"
#include "../../arch/x86_64/memory.h"

extern void kprintf(const char *fmt, ...);

#define VMM_VMALLOC_BASE 0xFFFFA00000000000ULL
#define VMM_VMALLOC_SIZE (128ULL << 30)
#define VMM_SIZE_MAX ((size_t)-1)

static bitmap_t vmm_bitmap;
static bool vmm_initialized = false;
static size_t vmm_total_pages = 0;
static size_t vmm_allocated_pages = 0;

static size_t vmm_lock_depth = 0;
static uint64_t vmm_lock_flags = 0;

static void vmm_lock(void)
{
    uint64_t flags;

    __asm__ volatile(
        "pushfq\n"
        "popq %0\n"
        "cli"
        : "=r"(flags)
        :
        : "memory"
    );

    if (vmm_lock_depth == 0) {
        vmm_lock_flags = flags;
    }

    vmm_lock_depth++;
}

static void vmm_unlock(void)
{
    if (vmm_lock_depth == 0) {
        return;
    }

    vmm_lock_depth--;

    if (vmm_lock_depth == 0) {
        uint64_t flags = vmm_lock_flags;

        __asm__ volatile(
            "pushq %0\n"
            "popfq"
            :
            : "r"(flags)
            : "memory"
        );
    }
}

static size_t vmm_bytes_to_pages(size_t bytes)
{
    if (bytes == 0) {
        return 0;
    }

    if (bytes > VMM_SIZE_MAX - ARCH_PAGE_SIZE) {
        return 0;
    }

    return (bytes + ARCH_PAGE_SIZE - 1) / ARCH_PAGE_SIZE;
}

static size_t vmm_align_to_pages(size_t align)
{
    if (align == 0 || align <= ARCH_PAGE_SIZE) {
        return 1;
    }

    uint64_t a = arch_page_align_up((uint64_t)align);

    if (a == UINT64_MAX || a < ARCH_PAGE_SIZE) {
        return 1;
    }

    uint64_t pages = a / ARCH_PAGE_SIZE;

    if (pages == 0) {
        return 1;
    }

    if (pages > VMM_SIZE_MAX) {
        return VMM_SIZE_MAX;
    }

    return (size_t)pages;
}

static uint64_t vmm_page_virt(size_t page)
{
    return VMM_VMALLOC_BASE + (uint64_t)page * ARCH_PAGE_SIZE;
}

static size_t vmm_virt_page(uint64_t virt)
{
    return (size_t)((virt - VMM_VMALLOC_BASE) / ARCH_PAGE_SIZE);
}

static bool vmm_in_region(uint64_t virt)
{
    return virt >= VMM_VMALLOC_BASE &&
           virt < VMM_VMALLOC_BASE + VMM_VMALLOC_SIZE;
}

static uint64_t vmm_flags_to_pte(uint32_t flags)
{
    uint64_t pte = PTE_PRESENT;

    if (flags & VMM_FLAG_WRITE) {
        pte |= PTE_WRITABLE;
    }

    if (flags & VMM_FLAG_USER) {
        pte |= PTE_USER;
    }

    if (flags & VMM_FLAG_NX) {
        pte |= PTE_NX;
    }

    if (flags & VMM_FLAG_DEVICE) {
        pte |= PTE_CACHE_DISABLE | PTE_WRITE_THROUGH | PTE_NX;
    }

    return pte;
}

bool vmm_init(void)
{
    if (vmm_initialized) {
        return true;
    }

    if (!pmm_ready()) {
        return false;
    }

    paging_enable_nx();

    vmm_total_pages = (size_t)(VMM_VMALLOC_SIZE / ARCH_PAGE_SIZE);

    size_t bitmap_bytes = bitmap_bytes_for_bits(vmm_total_pages);

    uint64_t bitmap_phys = 0;

    if (!pmm_alloc_bytes(bitmap_bytes, &bitmap_phys)) {
        return false;
    }

    if (!bitmap_init_phys(&vmm_bitmap, bitmap_phys, vmm_total_pages)) {
        return false;
    }

    bitmap_fill(&vmm_bitmap, false);

    vmm_allocated_pages = 0;
    vmm_initialized = true;

    return true;
}

bool vmm_ready(void)
{
    return vmm_initialized;
}

bool vmm_alloc_aligned(size_t bytes,
                       size_t align,
                       uint32_t flags,
                       uint64_t *out_virt)
{
    if (!vmm_initialized || out_virt == NULL || bytes == 0) {
        return false;
    }

    size_t pages = vmm_bytes_to_pages(bytes);
    size_t align_pages = vmm_align_to_pages(align);

    if (pages == 0 || align_pages == 0) {
        return false;
    }

    uint64_t pte = vmm_flags_to_pte(flags);

    vmm_lock();

    size_t start = bitmap_alloc_range(&vmm_bitmap, pages, align_pages);

    if (start == BITMAP_INVALID) {
        vmm_unlock();
        return false;
    }

    size_t mapped = 0;
    bool ok = true;

    for (size_t i = 0; i < pages; i++) {
        uint64_t phys = 0;

        bool got = (flags & VMM_FLAG_ZERO)
                       ? pmm_alloc_zero_frame(&phys)
                       : pmm_alloc_frame(&phys);

        if (!got) {
            ok = false;
            break;
        }

        uint64_t virt = vmm_page_virt(start + i);

        if (!paging_map_page(virt, phys, pte)) {
            pmm_free_frame(phys);
            ok = false;
            break;
        }

        mapped++;
    }

    if (!ok) {
        for (size_t i = 0; i < mapped; i++) {
            uint64_t virt = vmm_page_virt(start + i);

            if (paging_is_mapped(virt)) {
                uint64_t phys = paging_translate(virt);
                paging_unmap_page(virt);
                pmm_free_frame(phys);
            }
        }

        bitmap_free_range(&vmm_bitmap, start, pages);
        vmm_unlock();
        return false;
    }

    vmm_allocated_pages += pages;

    vmm_unlock();

    *out_virt = vmm_page_virt(start);

    return true;
}

bool vmm_alloc(size_t bytes, uint32_t flags, uint64_t *out_virt)
{
    return vmm_alloc_aligned(bytes, ARCH_PAGE_SIZE, flags, out_virt);
}

bool vmm_map_device(uint64_t phys, size_t len, uint64_t *out_virt)
{
    if (!vmm_initialized || out_virt == NULL || len == 0) {
        return false;
    }

    if (!arch_is_page_aligned(phys) || !arch_is_page_aligned(len)) {
        return false;
    }

    size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

    if (pages == 0) {
        return false;
    }

    uint64_t pte = vmm_flags_to_pte(VMM_FLAG_WRITE | VMM_FLAG_DEVICE);

    vmm_lock();

    size_t start = bitmap_alloc_range(&vmm_bitmap, pages, 1);

    if (start == BITMAP_INVALID) {
        vmm_unlock();
        return false;
    }

    size_t mapped = 0;
    bool ok = true;

    for (size_t i = 0; i < pages; i++) {
        uint64_t virt = vmm_page_virt(start + i);
        uint64_t frame_phys = phys + (uint64_t)i * ARCH_PAGE_SIZE;

        if (!paging_map_page(virt, frame_phys, pte)) {
            ok = false;
            break;
        }

        mapped++;
    }

    if (!ok) {
        for (size_t i = 0; i < mapped; i++) {
            paging_unmap_page(vmm_page_virt(start + i));
        }

        bitmap_free_range(&vmm_bitmap, start, pages);
        vmm_unlock();
        return false;
    }

    vmm_allocated_pages += pages;

    vmm_unlock();

    *out_virt = vmm_page_virt(start);

    return true;
}

bool vmm_free(uint64_t virt, size_t bytes)
{
    if (!vmm_initialized) {
        return false;
    }

    if (bytes == 0) {
        return true;
    }

    if (!arch_is_page_aligned(virt) || !vmm_in_region(virt)) {
        return false;
    }

    size_t pages = vmm_bytes_to_pages(bytes);
    size_t start = vmm_virt_page(virt);

    if (start >= vmm_total_pages || pages > vmm_total_pages - start) {
        return false;
    }

    vmm_lock();

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = vmm_page_virt(start + i);

        if (paging_is_mapped(v)) {
            uint64_t phys = paging_translate(v);
            paging_unmap_page(v);
            pmm_free_frame(phys);
        }
    }

    bitmap_free_range(&vmm_bitmap, start, pages);

    if (vmm_allocated_pages >= pages) {
        vmm_allocated_pages -= pages;
    } else {
        vmm_allocated_pages = 0;
    }

    vmm_unlock();

    return true;
}

bool vmm_unmap_device(uint64_t virt, size_t len)
{
    if (!vmm_initialized) {
        return false;
    }

    if (len == 0) {
        return true;
    }

    if (!arch_is_page_aligned(virt) || !vmm_in_region(virt)) {
        return false;
    }

    size_t pages = vmm_bytes_to_pages(len);
    size_t start = vmm_virt_page(virt);

    if (start >= vmm_total_pages || pages > vmm_total_pages - start) {
        return false;
    }

    vmm_lock();

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = vmm_page_virt(start + i);

        if (paging_is_mapped(v)) {
            paging_unmap_page(v);
        }
    }

    bitmap_free_range(&vmm_bitmap, start, pages);

    if (vmm_allocated_pages >= pages) {
        vmm_allocated_pages -= pages;
    } else {
        vmm_allocated_pages = 0;
    }

    vmm_unlock();

    return true;
}

uint64_t vmm_translate(uint64_t virt)
{
    if (!vmm_initialized) {
        return VMM_INVALID;
    }

    return paging_translate(virt);
}

size_t vmm_stat_total_pages(void)
{
    if (!vmm_initialized) {
        return 0;
    }

    return vmm_total_pages;
}

size_t vmm_stat_free_pages(void)
{
    if (!vmm_initialized) {
        return 0;
    }

    if (vmm_allocated_pages >= vmm_total_pages) {
        return 0;
    }

    return vmm_total_pages - vmm_allocated_pages;
}

size_t vmm_stat_allocated_pages(void)
{
    if (!vmm_initialized) {
        return 0;
    }

    return vmm_allocated_pages;
}

uint64_t vmm_stat_total_bytes(void)
{
    return (uint64_t)vmm_stat_total_pages() * ARCH_PAGE_SIZE;
}

uint64_t vmm_stat_free_bytes(void)
{
    return (uint64_t)vmm_stat_free_pages() * ARCH_PAGE_SIZE;
}

void vmm_dump(void)
{
    if (!vmm_initialized) {
        kprintf("VMM: not initialized\n");
        return;
    }

    kprintf("VMM:\n");
    kprintf("  base: 0x%llx\n", (unsigned long long)VMM_VMALLOC_BASE);
    kprintf("  total pages: %llu\n", (unsigned long long)vmm_stat_total_pages());
    kprintf("  free pages: %llu\n", (unsigned long long)vmm_stat_free_pages());
    kprintf("  allocated pages: %llu\n", (unsigned long long)vmm_stat_allocated_pages());
    kprintf("  total: %llu MiB\n", (unsigned long long)(vmm_stat_total_bytes() >> 20));
    kprintf("  free: %llu MiB\n", (unsigned long long)(vmm_stat_free_bytes() >> 20));
}