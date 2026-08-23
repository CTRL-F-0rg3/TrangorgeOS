#include "address_space.h"
#include "../alloc/physical/pmm.h"
#include "../alloc/heap/heap.h"
#include "../arch/x86_64/memory.h"
#include "../core/range.h"
#include "../core/smp_lock.h"

#define USER_ADDR_MIN  0x0000000000001000ULL
#define USER_MMAP_BASE 0x0000200000000000ULL
#define USER_MMAP_MAX  0x0000700000000000ULL
#define USER_STACK_TOP 0x00007FFF00000000ULL
#define USER_STACK_SIZE (128ULL * 1024)
#define USER_BRK_BASE  0x0000000001000000ULL
#define USER_BRK_MAX   0x0000000010000000ULL

static bool aspace_ready = false;

/*
 * P0.1: zastąpione rzeczywistą blokadą SMP (ticket lock + irqsave) —
 * dawny `pushfq; cli` + lokalny licznik chronił tylko przed przerwaniem
 * na BIEŻĄCYM CPU, nie przed drugim rdzeniem modyfikującym listę VMA
 * współbieżnie. `as_lock`/`as_unlock` zachowują dotychczasową semantykę
 * (w tym bezpieczną rekurencję, potrzebną np. w `aspace_map_at`, który
 * woła `aspace_unmap` trzymając już blokadę).
 */
static smp_ticket_lock_t as_smp_lock = SMP_TICKET_LOCK_INIT;

static void as_lock(void)
{
	smp_lock_acquire(&as_smp_lock);
}

static void as_unlock(void)
{
	smp_lock_release(&as_smp_lock);
}

bool aspace_subsystem_init(void)
{
	if (aspace_ready) {
	    return true;
	}

	if (!paging_subsystem_init()) {
	    return false;
	}

	aspace_ready = true;

	return true;
}

/*
 * Waliduje i wyrównuje (do granic strony) dowolny zakres user-space
 * podany jako (addr, len). Zastępuje dawne ręczne `a = align_down(addr)`
 * / `b = align_up(addr + len)`, które nie sprawdzały overflow — patrz
 * P0.3 w planie ulepszeń MM. Zwraca half-open [*out_start, *out_end).
 */
static bool user_range_ok(uint64_t addr, uint64_t len,
	                      uint64_t *out_start, uint64_t *out_end)
{
	return range_from_addr_len(addr, len, ARCH_PAGE_SIZE,
	                           USER_ADDR_MIN, USER_STACK_TOP,
	                           true, out_start, out_end);
}

static bool map_anon_pages(proc_aspace_t *pa,
	                       uint64_t start,
	                       uint64_t end,
	                       uint32_t prot)
{
	uint64_t mapped_end = start;

	for (uint64_t v = start; v < end; v += ARCH_PAGE_SIZE) {
	    uint64_t phys = 0;

	    if (!pmm_alloc_zero_frame(&phys)) {
	        goto rollback;
	    }

	    if (!paging_aspace_map(pa->as, v, phys, ARCH_PAGE_SIZE,
	                           prot | PROT_USER)) {
	        pmm_free_frame(phys);
	        goto rollback;
	    }

	    mapped_end = v + ARCH_PAGE_SIZE;
	}

	return true;

rollback:
	for (uint64_t v = start; v < mapped_end; v += ARCH_PAGE_SIZE) {
	    uint64_t phys = paging_aspace_translate(pa->as, v);

	    paging_aspace_unmap(pa->as, v, ARCH_PAGE_SIZE);

	    if (phys != UINT64_MAX) {
	        pmm_free_frame(phys);
	    }
	}

	return false;
}

static void unmap_pages_free(proc_aspace_t *pa, uint64_t start, uint64_t end)
{
	for (uint64_t v = start; v < end; v += ARCH_PAGE_SIZE) {
	    uint64_t phys = paging_aspace_translate(pa->as, v);

	    paging_aspace_unmap(pa->as, v, ARCH_PAGE_SIZE);

	    if (phys != UINT64_MAX) {
	        pmm_free_frame(phys);
	    }
	}
}

static void vma_insert(proc_aspace_t *pa, vma_t *v)
{
	vma_t **p = &pa->vmas;

	while (*p != NULL && (*p)->start < v->start) {
	    p = &(*p)->next;
	}

	v->next = *p;
	*p = v;
}

static void vma_remove(proc_aspace_t *pa, vma_t *v)
{
	vma_t **p = &pa->vmas;

	while (*p != NULL) {
	    if (*p == v) {
	        *p = v->next;
	        return;
	    }

	    p = &(*p)->next;
	}
}

vma_t *aspace_vma_find(proc_aspace_t *pa, uint64_t addr)
{
	vma_t *v = pa->vmas;

	while (v != NULL) {
	    if (addr >= v->start && addr < v->end) {
	        return v;
	    }

	    v = v->next;
	}

	return NULL;
}

static uint64_t find_gap(proc_aspace_t *pa, uint64_t len, uint64_t hint)
{
	uint64_t cand;

	if (hint != 0 && hint >= USER_MMAP_BASE) {
	    cand = arch_page_align_up(hint);
	} else {
	    cand = USER_MMAP_BASE;
	}

	if (cand > USER_MMAP_MAX - len) {
	    return 0;
	}

	vma_t *v = pa->vmas;

	while (v != NULL) {
	    if (v->end <= cand) {
	        v = v->next;
	        continue;
	    }

	    if (cand <= USER_MMAP_MAX - len && cand + len <= v->start) {
	        return cand;
	    }

	    cand = arch_page_align_up(v->end);

	    if (cand > USER_MMAP_MAX - len) {
	        return 0;
	    }

	    v = v->next;
	}

	return cand;
}

proc_aspace_t *aspace_create(void)
{
	if (!aspace_ready) {
	    return NULL;
	}

	proc_aspace_t *pa = (proc_aspace_t *)heap_zalloc(sizeof(proc_aspace_t));

	if (pa == NULL) {
	    return NULL;
	}

	pa->as = paging_aspace_create();

	if (pa->as == NULL) {
	    heap_free(pa);
	    return NULL;
	}

	pa->brk_base = USER_BRK_BASE;
	pa->brk = USER_BRK_BASE;
	pa->brk_max = USER_BRK_MAX;

	uint64_t stack_start = USER_STACK_TOP - USER_STACK_SIZE;

	vma_t *sv = (vma_t *)heap_alloc(sizeof(vma_t));

	if (sv == NULL) {
	    paging_aspace_destroy(pa->as);
	    heap_free(pa);
	    return NULL;
	}

	sv->start = stack_start;
	sv->end = USER_STACK_TOP;
	sv->prot = PROT_READ | PROT_WRITE;
	sv->flags = VMA_FLAG_ANON | VMA_FLAG_PRIVATE;
	sv->next = NULL;

	pa->vmas = sv;

	if (!map_anon_pages(pa, stack_start, USER_STACK_TOP, sv->prot)) {
	    heap_free(sv);
	    paging_aspace_destroy(pa->as);
	    heap_free(pa);
	    return NULL;
	}

	return pa;
}

void aspace_destroy(proc_aspace_t *pa)
{
	if (pa == NULL) {
	    return;
	}

	as_lock();

	while (pa->vmas != NULL) {
	    vma_t *v = pa->vmas;

	    unmap_pages_free(pa, v->start, v->end);

	    pa->vmas = v->next;
	    heap_free(v);
	}

	paging_aspace_destroy(pa->as);

	heap_free(pa);

	as_unlock();
}

address_space_t *aspace_paging_handle(proc_aspace_t *pa)
{
	if (pa == NULL) {
	    return NULL;
	}

	return pa->as;
}

uint64_t aspace_map_anon(proc_aspace_t *pa, uint64_t hint, size_t len, uint32_t prot)
{
	if (pa == NULL || len == 0) {
	    return 0;
	}

	uint64_t bytes = arch_page_align_up((uint64_t)len);

	as_lock();

	uint64_t at = find_gap(pa, bytes, hint);

	if (at == 0) {
	    as_unlock();
	    return 0;
	}

	vma_t *v = (vma_t *)heap_alloc(sizeof(vma_t));

	if (v == NULL) {
	    as_unlock();
	    return 0;
	}

	v->start = at;
	v->end = at + bytes;
	v->prot = prot;
	v->flags = VMA_FLAG_ANON | VMA_FLAG_PRIVATE;

	vma_insert(pa, v);

	if (!map_anon_pages(pa, at, at + bytes, prot)) {
	    vma_remove(pa, v);
	    heap_free(v);
	    as_unlock();
	    return 0;
	}

	as_unlock();

	return at;
}

uint64_t aspace_map_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
{
	if (pa == NULL || len == 0) {
	    return 0;
	}

	uint64_t a, b;

	if (!user_range_ok(addr, (uint64_t)len, &a, &b)) {
	    return 0;
	}

	as_lock();

	aspace_unmap(pa, a, b - a);

	vma_t *v = (vma_t *)heap_alloc(sizeof(vma_t));

	if (v == NULL) {
	    as_unlock();
	    return 0;
	}

	v->start = a;
	v->end = b;
	v->prot = prot;
	v->flags = VMA_FLAG_ANON | VMA_FLAG_PRIVATE;

	vma_insert(pa, v);

	if (!map_anon_pages(pa, a, b, prot)) {
	    vma_remove(pa, v);
	    heap_free(v);
	    as_unlock();
	    return 0;
	}

	as_unlock();

	return a;
}

bool aspace_unmap(proc_aspace_t *pa, uint64_t addr, size_t len)
{
	if (pa == NULL || len == 0) {
	    return false;
	}

	uint64_t a, b;

	if (!user_range_ok(addr, (uint64_t)len, &a, &b)) {
	    return false;
	}

	as_lock();

	while (true) {
	    vma_t *v = pa->vmas;
	    vma_t *hit = NULL;

	    while (v != NULL) {
	        if (v->start < b && v->end > a) {
	            hit = v;
	            break;
	        }

	        v = v->next;
	    }

	    if (hit == NULL) {
	        break;
	    }

	    uint64_t s = hit->start > a ? hit->start : a;
	    uint64_t e = hit->end < b ? hit->end : b;

	    bool needs_split = (hit->start < s && hit->end > e);
	    vma_t *tail = NULL;

	    if (needs_split) {
	        /*
	         * P1 (mmap/VMA — atomowe rozbijanie VMA): metadane
	         * rozbicia MUSZĄ być zaalokowane PRZED faktycznym
	         * odmapowaniem/zwolnieniem stron w [s,e). Wcześniej
	         * `heap_alloc()` był wołany PO `unmap_pages_free()` —
	         * porażka alokacji (presja pamięciowa) zwracała `false`,
	         * zostawiając `hit` NIEZMIENIONE (nadal obejmujące cały
	         * stary zakres [hit->start, hit->end)), mimo że środkowy
	         * fragment [s,e) był już faktycznie odmapowany z tablic
	         * stron, a jego ramki fizyczne oddane z powrotem do PMM
	         * (i mogły trafić do zupełnie innego przydziału). Metadane
	         * VMA kłamałyby więc, że zakres jest wciąż zmapowany, choć
	         * fizycznie już nie był — kolejny dostęp do tego adresu
	         * mógłby np. zostać błędnie obsłużony przez logikę page
	         * faulta polegającą na tym, że VMA istnieje.
	         */
	        tail = (vma_t *)heap_alloc(sizeof(vma_t));

	        if (tail == NULL) {
	            as_unlock();
	            return false;
	        }
	    }

	    unmap_pages_free(pa, s, e);

	    if (needs_split) {
	        tail->start = e;
	        tail->end = hit->end;
	        tail->prot = hit->prot;
	        tail->flags = hit->flags;
	        tail->next = hit->next;

	        hit->next = tail;
	        hit->end = s;
	    } else if (hit->start < s) {
	        hit->end = s;
	    } else if (hit->end > e) {
	        hit->start = e;
	    } else {
	        vma_remove(pa, hit);
	        heap_free(hit);
	    }
	}

	as_unlock();

	return true;
}

bool aspace_protect_checked(proc_aspace_t *pa, uint64_t addr, size_t len,
	                        uint32_t checked_prot,
	                        uint32_t apply_prot,
	                        bool (*allowed)(uint32_t old_prot,
	                                        uint32_t checked_prot))
{
	if (pa == NULL || len == 0) {
	    return false;
	}

	uint64_t a, b;

	if (!user_range_ok(addr, (uint64_t)len, &a, &b)) {
	    return false;
	}

	as_lock();

	/*
	 * P1 (mmap/VMA): wyszukanie VMA, sprawdzenie dozwolonej zmiany
	 * uprawnien i sama zmiana sa TERAZ jedna, nieprzerywalna sekcja pod
	 * jednym trzymaniem `as_smp_lock`. Wczesniejszy `mprotect()` w
	 * process/mmap.c wolal `aspace_vma_find()` PRZED wejsciem w jakakolwiek
	 * blokade, po czym czytal `v->prot` z niezablokowanego wskaznika —
	 * scisly use-after-free, gdyby inny rdzen zwolnil ta VMA przez
	 * `aspace_unmap()`/`munmap()` w tym samym momencie.
	 */
	vma_t *v = aspace_vma_find(pa, a);

	if (v == NULL || b > v->end) {
	    as_unlock();
	    return false;
	}

	if (allowed != NULL && !allowed(v->prot, checked_prot)) {
	    as_unlock();
	    return false;
	}

	if (!paging_aspace_protect(pa->as, a, b - a, apply_prot)) {
	    as_unlock();
	    return false;
	}

	v->prot = apply_prot;

	as_unlock();

	return true;
}

bool aspace_protect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
{
	return aspace_protect_checked(pa, addr, len, prot, prot, NULL);
}

uint64_t aspace_stack_base(void)
{
	return USER_STACK_TOP - USER_STACK_SIZE;
}

uint64_t aspace_reserve_at(proc_aspace_t *pa,
	                       uint64_t addr,
	                       size_t len,
	                       uint32_t flags)
{
	if (pa == NULL || len == 0) {
	    return 0;
	}

	uint64_t a, b;

	if (!user_range_ok(addr, (uint64_t)len, &a, &b)) {
	    return 0;
	}

	as_lock();

	vma_t *v = pa->vmas;

	while (v != NULL) {
	    if (v->start < b && v->end > a) {
	        as_unlock();
	        return 0;
	    }

	    v = v->next;
	}

	vma_t *nv = (vma_t *)heap_alloc(sizeof(vma_t));

	if (nv == NULL) {
	    as_unlock();
	    return 0;
	}

	nv->start = a;
	nv->end = b;
	nv->prot = 0;
	nv->flags = flags;

	vma_insert(pa, nv);

	as_unlock();

	return a;
}

uint64_t aspace_brk(proc_aspace_t *pa, uint64_t new_brk)
{
	if (pa == NULL) {
	    return 0;
	}

	as_lock();

	if (new_brk == 0) {
	    uint64_t cur = pa->brk;
	    as_unlock();
	    return cur;
	}

	if (new_brk < pa->brk_base || new_brk > pa->brk_max) {
	    as_unlock();
	    return pa->brk;
	}

	uint64_t nb = arch_page_align_up(new_brk);

	if (nb > pa->brk) {
	    /*
	     * P0.4: najpierw mapujemy strony, a metadane VMA/`brk` zatwierdzamy
	     * dopiero po pełnym sukcesie. Wcześniej VMA było rozszerzane PRZED
	     * mapowaniem, więc częściowy błąd `map_anon_pages()` zostawiał
	     * VMA większe niż faktycznie zmapowane strony (rozjazd metadanych
	     * i tablic stron). `map_anon_pages()` sam wykonuje rollback
	     * częściowo zmapowanych stron przy błędzie, więc w razie porażki
	     * tutaj nic jeszcze nie zostało zmienione w VMA/`brk`.
	     */
	    if (!map_anon_pages(pa, pa->brk, nb, PROT_READ | PROT_WRITE)) {
	        as_unlock();
	        return pa->brk;
	    }

	    vma_t *v = aspace_vma_find(pa, pa->brk_base);

	    if (v == NULL) {
	        v = (vma_t *)heap_alloc(sizeof(vma_t));

	        if (v == NULL) {
	            /* Strony zmapowane, ale brak metadanych — wycofaj mapowanie,
	             * żeby stan pozostał spójny (brak VMA opisującego te strony). */
	            unmap_pages_free(pa, pa->brk, nb);
	            as_unlock();
	            return pa->brk;
	        }

	        v->start = pa->brk_base;
	        v->end = nb;
	        v->prot = PROT_READ | PROT_WRITE;
	        v->flags = VMA_FLAG_ANON | VMA_FLAG_PRIVATE;

	        vma_insert(pa, v);
	    } else {
	        v->end = nb;
	    }

	    pa->brk = nb;
	} else if (nb < pa->brk) {
	    aspace_unmap(pa, nb, pa->brk - nb);
	    pa->brk = nb;
	}

	uint64_t cur = pa->brk;

	as_unlock();

	return cur;
}