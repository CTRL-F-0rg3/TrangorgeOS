#include "paging.h"
#include "pml.h"
#include "tlb.h"
#include "../arch/x86_64/paging.h"
#include "../arch/x86_64/memory.h"
#include "../alloc/physical/pmm.h"
#include "../alloc/heap/heap.h"
#include "../arch/x86_64/tlb.h"
#include "../core/range.h"
static uint64_t paging_kernel_pml4 = 0;
static bool paging_subsystem_ready = false;

static uint64_t prot_to_pte(uint32_t prot)
{
    uint64_t pte = PTE_PRESENT;

    if (prot & PROT_WRITE) {
        pte |= PTE_WRITABLE;
    }

    if (!(prot & PROT_EXEC)) {
        pte |= PTE_NX;
    }

    if (prot & PROT_USER) {
        pte |= PTE_USER;
    }

    if (prot & PROT_DEVICE) {
        pte |= PTE_CACHE_DISABLE | PTE_WRITE_THROUGH | PTE_NX;
    }

    return pte;
}

/*
 * Waliduje, ze `addr`/`len` sa scisle wyrownane do strony i ze `addr+len`
 * nie przepelnia u64. Uzywana defensywnie na tej (najnizszej) warstwie
 * przez paging_aspace_map/unmap/protect — obecni wywolujacy (przez
 * address_space.c) juz waliduja zakres wczesniej (P0.3), ale ta funkcja
 * jest tez czescia publicznego API `paging.h`, wiec przyszli wywolujacy
 * (np. paging_kernel_map/unmap, obecnie bez zadnego wywolujacego w
 * drzewie) dostaja te sama gwarancje bez polegania na warstwie wyzej.
 */
static bool page_range_ok(uint64_t addr, size_t len)
{
    uint64_t start, end;

    if (!range_from_addr_len(addr, (uint64_t)len, ARCH_PAGE_SIZE,
                             0, UINT64_MAX, false, &start, &end)) {
        return false;
    }

    return start == addr && end == addr + (uint64_t)len;
}

bool paging_subsystem_init(void)
{
    if (paging_subsystem_ready) {
        return true;
    }

    paging_kernel_pml4 = paging_read_cr3();
    paging_subsystem_ready = true;

    return true;
}

static void free_tables(uint64_t table_phys, int level)
{
    uint64_t *table = pml_table_ptr(table_phys);

    if (level > PML_LEVEL_PT) {
        for (size_t i = 0; i < PML_ENTRIES; i++) {
            uint64_t e = table[i];

            if (pml_entry_present(e) && !pml_entry_large(e)) {
                free_tables(pml_entry_addr(e), level - 1);
            }
        }
    }

    pmm_free_frame(table_phys);
}

address_space_t *paging_aspace_create(void)
{
    if (!paging_subsystem_ready) {
        return NULL;
    }

    uint64_t pml4_phys = 0;

    if (!pmm_alloc_zero_frame(&pml4_phys)) {
        return NULL;
    }

    uint64_t *new_pml4 = pml_table_ptr(pml4_phys);
    uint64_t *kernel_pml4 = pml_table_ptr(paging_kernel_pml4);

    for (size_t i = 256; i < 512; i++) {
        new_pml4[i] = kernel_pml4[i];
    }

    address_space_t *as = (address_space_t *)heap_alloc(sizeof(address_space_t));

    if (as == NULL) {
        pmm_free_frame(pml4_phys);
        return NULL;
    }

    as->pml4_phys = pml4_phys;
    as->kernel = false;

    return as;
}

void paging_aspace_destroy(address_space_t *as)
{
    if (as == NULL || as->kernel) {
        return;
    }

    uint64_t *pml4 = pml_table_ptr(as->pml4_phys);

    for (size_t i = 0; i < 256; i++) {
        uint64_t e = pml4[i];

        if (pml_entry_present(e) && !pml_entry_large(e)) {
            free_tables(pml_entry_addr(e), PML_LEVEL_PDPT);
        }
    }

    pmm_free_frame(as->pml4_phys);

    heap_free(as);
}

void paging_aspace_switch(address_space_t *as)
{
    if (as == NULL) {
        return;
    }

    paging_write_cr3(as->pml4_phys);
    tlb_flush_all();
}

uint64_t paging_aspace_cr3(const address_space_t *as)
{
    if (as == NULL) {
        return paging_kernel_pml4;
    }

    return as->pml4_phys;
}

bool paging_aspace_map(address_space_t *as,
                       uint64_t virt,
                       uint64_t phys,
                       size_t len,
                       uint32_t prot)
{
    if (as == NULL || len == 0) {
        return false;
    }

    if (!page_range_ok(virt, len) || !page_range_ok(phys, len)) {
        return false;
    }

    uint64_t pml4 = as->pml4_phys;
    uint64_t pte = prot_to_pte(prot);

    size_t pages = (size_t)(len / ARCH_PAGE_SIZE);
    size_t mapped = 0;

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;
        uint64_t p = phys + (uint64_t)i * ARCH_PAGE_SIZE;

        if (!paging_map_page_in(pml4, v, p, pte)) {
            for (size_t j = 0; j < mapped; j++) {
                paging_unmap_page_in(pml4, virt + (uint64_t)j * ARCH_PAGE_SIZE);
            }

            return false;
        }

        mapped++;
    }

    return true;
}

bool paging_aspace_unmap(address_space_t *as, uint64_t virt, size_t len)
{
    if (as == NULL || len == 0) {
        return false;
    }

    if (!page_range_ok(virt, len)) {
        return false;
    }

    uint64_t pml4 = as->pml4_phys;

    tlb_batch_t batch;
    tlb_batch_begin(&batch);

    size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

        if (paging_unmap_page_in(pml4, v)) {
            tlb_batch_add(&batch, v);
        }
    }

    tlb_batch_commit(&batch);

    return true;
}

bool paging_aspace_protect(address_space_t *as,
                           uint64_t virt,
                           size_t len,
                           uint32_t prot)
{
    if (as == NULL || len == 0) {
        return false;
    }

    if (!page_range_ok(virt, len)) {
        return false;
    }

    uint64_t pml4 = as->pml4_phys;
    uint64_t pte = prot_to_pte(prot);

    size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

    /*
     * P1 (paging/mapowania — ten sam problem co w mapping_protect_range()
     * z alloc/virtual/mapping.c, tu naprawiony w PRAWDZIWEJ ścieżce
     * używanej przez aspace_protect(): brak walidacji przed mutacją
     * oznaczał, że natrafienie na niezmapowaną stronę w połowie zakresu
     * przerywało pętlę PO ZMIANIE flag prefiksu i zwracało false —
     * wywołujący (aspace_protect) poprawnie nie aktualizował v->prot,
     * ale tablice stron już miały nowe uprawnienia na części zakresu.
     * Rozdzielenie na dwa przebiegi (walidacja, potem zapis) czyni
     * operację atomową w normalnym, jednowątkowym względem tej
     * przestrzeni adresowej przypadku.
     */
    for (size_t i = 0; i < pages; i++) {
        uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

        if (!paging_is_mapped_in(pml4, v)) {
            return false;
        }
    }

    tlb_batch_t batch;
    tlb_batch_begin(&batch);

    for (size_t i = 0; i < pages; i++) {
        uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

        if (!paging_set_flags_in(pml4, v, pte)) {
            /* Nie powinno wystapic po walidacji powyzej — patrz analogiczny
             * komentarz w mapping_protect_range(). */
            tlb_batch_commit(&batch);
            return false;
        }

        tlb_batch_add(&batch, v);
    }

    tlb_batch_commit(&batch);

    return true;
}

uint64_t paging_aspace_translate(address_space_t *as, uint64_t virt)
{
    if (as == NULL) {
        return paging_translate_in(paging_kernel_pml4, virt);
    }

    return paging_translate_in(as->pml4_phys, virt);
}

bool paging_kernel_map(uint64_t virt, uint64_t phys, size_t len, uint32_t prot)
{
    if (!paging_subsystem_ready) {
        return false;
    }

    return paging_aspace_map((address_space_t *)&(address_space_t){
                                 .pml4_phys = paging_kernel_pml4,
                                 .kernel = true,
                             },
                             virt, phys, len, prot);
}

bool paging_kernel_unmap(uint64_t virt, size_t len)
{
    if (!paging_subsystem_ready) {
        return false;
    }

    return paging_aspace_unmap((address_space_t *)&(address_space_t){
                                   .pml4_phys = paging_kernel_pml4,
                                   .kernel = true,
                               },
                               virt, len);
}