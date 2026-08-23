#include "alloc_debug.h"
#include "leak.h"
#include "stats.h"
#include "../heap/heap.h"
#include "../physical/pmm.h"
#include "../virtual/vmm.h"
#include "../heap/buddy.h"
#include "../heap/slab.h"
#include "../../cache/cache.h"

extern void kprintf(const char *fmt, ...);

#define DBG_MAGIC 0x444247414C4C4F43ULL
#define DBG_POISON_ALLOC 0xAA
#define DBG_POISON_FREE  0xFF

typedef struct dbg_header {
	uint64_t magic;
	uint64_t size;
	uint64_t caller;
	/*
	 * Wcześniej zawsze 0 ("reserved"). Teraz: 1 jeśli ta alokacja została
	 * pomyślnie wpisana do tabeli leaków (leak_track_id() zwróciło true),
	 * 0 jeśli tabela była pełna w momencie alokacji. `dbg_free()` używa
	 * tego, żeby NIE wołać `leak_untrack()` (i nie zgłaszać fałszywego
	 * "double free") dla alokacji, których nigdy nie było w tabeli —
	 * patrz uzasadnienie w leak.h przy leak_track_id().
	 */
	uint64_t tracked;
	uint64_t alloc_id;
} dbg_header_t;

static uint64_t dbg_tail_canary(size_t size)
{
	return ~((uint64_t)size) ^ DBG_MAGIC;
}

static uint64_t *dbg_tail_ptr(void *user, size_t size)
{
	return (uint64_t *)((uint8_t *)user + size);
}

void *dbg_alloc(size_t size)
{
	if (size == 0) {
	    return NULL;
	}

	uint64_t caller = (uint64_t)(uintptr_t)__builtin_return_address(0);

	uint8_t *real =
	    (uint8_t *)heap_alloc(sizeof(dbg_header_t) + size + sizeof(uint64_t));

	if (real == NULL) {
	    return NULL;
	}

	dbg_header_t *h = (dbg_header_t *)real;

	h->magic = DBG_MAGIC;
	h->size = size;
	h->caller = caller;
	h->tracked = 0;
	h->alloc_id = 0;

	uint8_t *user = real + sizeof(dbg_header_t);

	for (size_t i = 0; i < size; i++) {
	    user[i] = DBG_POISON_ALLOC;
	}

	*dbg_tail_ptr(user, size) = dbg_tail_canary(size);

	/*
	 * Naprawa błędu (sekcja 5 planu — "brak cichego nieśledzenia
	 * alokacji po przekroczeniu limitu"): wcześniej wynik leak_track()
	 * był całkowicie ignorowany. Gdy tabela leaków (LEAK_MAX=2048) była
	 * pełna, alokacja i tak "udawała się" (heap_alloc już zwrócił
	 * pamięć), ale zostawała cicho nieśledzona. Przy odpowiadającym
	 * dbg_free() taki wskaźnik nie był w tabeli, więc leak_untrack()
	 * zwracał false, dbg_free() zgłaszał FAŁSZYWY alarm "double free or
	 * unknown ptr" i — co gorsza — WRACAŁ PRZED wywołaniem heap_free(),
	 * czyli pamięć nigdy nie była faktycznie zwalniana: prawdziwy,
	 * trwały wyciek wywołany samym przepełnieniem tabeli diagnostycznej.
	 * Teraz zapamiętujemy w nagłówku, czy śledzenie się powiodło, i
	 * dbg_free() odpowiednio dostosowuje swoją ścieżkę (patrz tam).
	 */
	uint64_t alloc_id = 0;

	if (leak_track_id(user, size, caller, &alloc_id)) {
	    h->tracked = 1;
	    h->alloc_id = alloc_id;
	}

	alloc_stats_note_alloc(size);

	return user;
}

void dbg_free(void *ptr)
{
	if (ptr == NULL) {
	    return;
	}

	dbg_header_t *h =
	    (dbg_header_t *)((uint8_t *)ptr - sizeof(dbg_header_t));

	if (h->magic != DBG_MAGIC) {
	    kprintf("dbg_free: bad magic at %p\n", ptr);
	    return;
	}

	size_t size = h->size;

	if (*dbg_tail_ptr(ptr, size) != dbg_tail_canary(size)) {
	    kprintf("dbg_free: buffer overflow at %p caller 0x%llx\n",
	            ptr,
	            (unsigned long long)h->caller);
	}

	/*
	 * Wołaj leak_untrack() TYLKO dla alokacji, które faktycznie zostały
	 * wpisane do tabeli przy dbg_alloc() (patrz komentarz przy
	 * `tracked` w definicji struktury i przy leak_track_id() w leak.h).
	 * Dla nieśledzonych alokacji (tabela była pełna) pomijamy ten
	 * krok — inaczej dostalibyśmy fałszywy "double free or unknown ptr"
	 * dla zwyczajnego, poprawnego pojedynczego zwolnienia. Prawdziwy
	 * double-free jest i tak wykrywany niezależnie od tabeli leaków:
	 * ta funkcja zeruje `h->magic` poniżej, więc DRUGIE wywołanie
	 * dbg_free() na tym samym wskaźniku zostanie odrzucone przez
	 * sprawdzenie magic na samej górze funkcji (`h->magic != DBG_MAGIC`).
	 */
	if (h->tracked) {
	    size_t leaked_size = 0;
	    uint64_t alloc_caller = 0;

	    if (!leak_untrack(ptr, &leaked_size, &alloc_caller)) {
	        kprintf("dbg_free: double free or unknown ptr %p\n", ptr);
	        return;
	    }
	}

	alloc_stats_note_free(size);

	h->magic = 0;

	uint8_t *real = (uint8_t *)h;
	size_t total = sizeof(dbg_header_t) + size + sizeof(uint64_t);

	for (size_t i = 0; i < total; i++) {
	    real[i] = DBG_POISON_FREE;
	}

	heap_free(real);
}

size_t dbg_usable_size(void *ptr)
{
	if (ptr == NULL) {
	    return 0;
	}

	dbg_header_t *h =
	    (dbg_header_t *)((uint8_t *)ptr - sizeof(dbg_header_t));

	if (h->magic != DBG_MAGIC) {
	    return 0;
	}

	return h->size;
}

bool dbg_verify(void *ptr)
{
	if (ptr == NULL) {
	    return false;
	}

	dbg_header_t *h =
	    (dbg_header_t *)((uint8_t *)ptr - sizeof(dbg_header_t));

	if (h->magic != DBG_MAGIC) {
	    return false;
	}

	return *dbg_tail_ptr(ptr, h->size) == dbg_tail_canary(h->size);
}

void mm_debug_dump(void)
{
	alloc_stats_dump();
	leak_dump();
	pmm_dump();
	vmm_dump();
	buddy_dump();
	slab_dump();
	cache_dump();
}