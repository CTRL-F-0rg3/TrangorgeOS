#include "page.h"
#include "../physical/pmm.h"
//#include "../physical/frame.h"
#include "../../arch/x86_64/memory.h"

extern void kprintf(const char *fmt, ...);

static page_t *page_descs = NULL;
static size_t page_desc_count = 0;
static bool page_initialized = false;

static size_t page_lock_depth = 0;
static uint64_t page_lock_flags = 0;

static void page_lock(void)
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

    if (page_lock_depth == 0) {
        page_lock_flags = flags;
    }

    page_lock_depth++;
}

static void page_unlock(void)
{
    if (page_lock_depth == 0) {
        return;
    }

    page_lock_depth--;

    if (page_lock_depth == 0) {
        uint64_t flags = page_lock_flags;

        __asm__ volatile(
            "pushq %0\n"
            "popfq"
            :
            : "r"(flags)
            : "memory"
        );
    }
}

static size_t page_phys_to_pfn(uint64_t phys)
{
    return (size_t)(phys / ARCH_PAGE_SIZE);
}

static bool page_ptr_valid(const page_t *page)
{
    if (page == NULL || page_descs == NULL) {
        return false;
    }

    size_t idx = (size_t)(page - page_descs);

    return idx < page_desc_count;
}

bool page_init(void)
{
    if (page_initialized) {
        return true;
    }

    if (!pmm_ready()) {
        kprintf("page_init: pmm not ready\n");
        return false;
    }

    const arch_mem_info_t *info = arch_memory_get();

    if (info == NULL) {
        kprintf("page_init: no arch mem info\n");
        return false;
    }

    uint64_t max_pfn = info->max_usable_address / ARCH_PAGE_SIZE;

    page_desc_count = (size_t)(max_pfn + 1);

    size_t bytes = page_desc_count * sizeof(page_t);

    kprintf("page_init: max_usable=0x%llx max_addr=0x%llx desc_count=%llu bytes=%llu\n",
            (unsigned long long)info->max_usable_address,
            (unsigned long long)info->max_address,
            (unsigned long long)page_desc_count,
            (unsigned long long)bytes);

    uint64_t descs_phys = 0;

    if (!pmm_alloc_bytes(bytes, &descs_phys)) {
        kprintf("page_init: pmm_alloc_bytes(%llu) FAILED\n",
                (unsigned long long)bytes);
        return false;
    }

    kprintf("page_init: descs_phys=0x%llx\n", (unsigned long long)descs_phys);

    page_descs = (page_t *)arch_phys_to_virt(descs_phys);

    for (size_t i = 0; i < page_desc_count; i++) {
        page_descs[i].refcount = 0;
        page_descs[i].type = PAGE_TYPE_FREE;
        page_descs[i].flags = 0;
        page_descs[i].reserved = 0;
    }

    page_initialized = true;

    return true;
}

bool page_ready(void)
{
    return page_initialized;
}

page_t *page_alloc(uint32_t type)
{
    if (!page_initialized) {
        return NULL;
    }

    uint64_t phys = 0;

    page_lock();

    if (!pmm_alloc_frame(&phys)) {
        page_unlock();
        return NULL;
    }

    page_t *p = &page_descs[page_phys_to_pfn(phys)];

    p->refcount = 1;
    p->type = type;
    p->flags = 0;
    p->reserved = 0;

    page_unlock();

    return p;
}

page_t *page_alloc_zero(uint32_t type)
{
    if (!page_initialized) {
        return NULL;
    }

    uint64_t phys = 0;

    page_lock();

    if (!pmm_alloc_zero_frame(&phys)) {
        page_unlock();
        return NULL;
    }

    page_t *p = &page_descs[page_phys_to_pfn(phys)];

    p->refcount = 1;
    p->type = type;
    p->flags = 0;
    p->reserved = 0;

    page_unlock();

    return p;
}

bool page_attach(uint64_t phys, uint32_t type)
{
    if (!page_initialized) {
        return false;
    }

    if (!arch_is_page_aligned(phys)) {
        return false;
    }

    size_t pfn = page_phys_to_pfn(phys);

    if (pfn >= page_desc_count) {
        return false;
    }

    page_lock();

    page_t *p = &page_descs[pfn];

    if (p->refcount != 0) {
        page_unlock();
        return false;
    }

    p->refcount = 1;
    p->type = type;

    page_unlock();

    return true;
}

bool page_get(page_t *page)
{
    if (!page_initialized || !page_ptr_valid(page)) {
        return false;
    }

    page_lock();

    if (page->refcount == 0 || page->refcount == UINT32_MAX) {
        page_unlock();
        return false;
    }

    page->refcount++;

    page_unlock();

    return true;
}

bool page_put(page_t *page)
{
    if (!page_initialized || !page_ptr_valid(page)) {
        return false;
    }

    page_lock();

    if (page->refcount == 0) {
        page_unlock();
        return false;
    }

    page->refcount--;

    if (page->refcount != 0) {
        page_unlock();
        return false;
    }

    size_t idx = (size_t)(page - page_descs);
    uint64_t phys = (uint64_t)idx * ARCH_PAGE_SIZE;

    page->type = PAGE_TYPE_FREE;
    page->flags = 0;

    page_unlock();

    pmm_free_frame(phys);

    return true;
}

uint64_t page_phys(const page_t *page)
{
    if (!page_ptr_valid(page)) {
        return UINT64_MAX;
    }

    size_t idx = (size_t)(page - page_descs);

    return (uint64_t)idx * ARCH_PAGE_SIZE;
}

void *page_kva(const page_t *page)
{
    uint64_t phys = page_phys(page);

    if (phys == UINT64_MAX) {
        return NULL;
    }

    return arch_phys_to_virt(phys);
}

size_t page_pfn(const page_t *page)
{
    if (!page_ptr_valid(page)) {
        return (size_t)-1;
    }

    return (size_t)(page - page_descs);
}

page_t *page_from_phys(uint64_t phys)
{
    if (!page_initialized || !arch_is_page_aligned(phys)) {
        return NULL;
    }

    size_t pfn = page_phys_to_pfn(phys);

    if (pfn >= page_desc_count) {
        return NULL;
    }

    return &page_descs[pfn];
}

page_t *page_from_pfn(size_t pfn)
{
    if (!page_initialized || pfn >= page_desc_count) {
        return NULL;
    }

    return &page_descs[pfn];
}

page_t *page_from_kva(const void *va)
{
    if (!page_initialized || va == NULL) {
        return NULL;
    }

    return page_from_phys(arch_virt_to_phys(va));
}

uint32_t page_refcount(const page_t *page)
{
    if (!page_ptr_valid(page)) {
        return 0;
    }

    return page->refcount;
}

uint32_t page_type(const page_t *page)
{
    if (!page_ptr_valid(page)) {
        return PAGE_TYPE_FREE;
    }

    return page->type;
}

void page_set_type(page_t *page, uint32_t type)
{
    if (!page_ptr_valid(page)) {
        return;
    }

    page->type = type;
}

size_t page_stat_total(void)
{
    if (!page_initialized) {
        return 0;
    }

    return page_desc_count;
}

size_t page_stat_referenced(void)
{
    if (!page_initialized) {
        return 0;
    }

    size_t count = 0;

    for (size_t i = 0; i < page_desc_count; i++) {
        if (page_descs[i].refcount != 0) {
            count++;
        }
    }

    return count;
}

void page_dump(void)
{
    if (!page_initialized) {
        kprintf("PAGE: not initialized\n");
        return;
    }

    kprintf("PAGE:\n");
    kprintf("  total pages: %llu\n", (unsigned long long)page_stat_total());
    kprintf("  referenced: %llu\n", (unsigned long long)page_stat_referenced());
}