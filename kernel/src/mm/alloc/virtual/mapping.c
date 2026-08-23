#include "mapping.h"
#include "../../arch/x86_64/paging.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/range.h"

static uint64_t mapping_kernel_pml4 = 0;
static bool mapping_initialized = false;

static uint64_t resolve_space(mapping_space_t space)
{
	if (space == MAPPING_KERNEL) {
	    return mapping_kernel_pml4;
	}

	return space;
}

void mapping_init(void)
{
	mapping_kernel_pml4 = paging_read_cr3();
	mapping_initialized = true;
}

mapping_space_t mapping_kernel_space(void)
{
	if (!mapping_initialized) {
	    mapping_init();
	}

	return mapping_kernel_pml4;
}

/*
 * Wspolna walidacja zakresu strona-wyrownanego, uzywana przez KAZDA
 * publiczna funkcje tego pliku (map/unmap/protect/copy). Zastepuje
 * dawne rozne kombinacje `arch_is_page_aligned()` bez sprawdzania
 * `addr + len` pod katem przepelnienia — do tej pory ochrone przed
 * overflow mial WYLACZNIE `mapping_map_range()` (przez `range_valid`),
 * podczas gdy `mapping_unmap_range()`, `mapping_protect_range()` i
 * `mapping_copy_range()` w ogole go nie sprawdzaly. To ta sama klasa
 * bledu co P0.3 w address_space.c, tylko w innym pliku — teraz
 * naprawiona przez ponowne uzycie tego samego helpera z core/range.c
 * (spelnia definicje ukonczenia Etapu A: "wszystkie publiczne API
 * zakresowe uzywaja helperow").
 *
 * Kontrakt: `addr` i `len` musza byc JUZ dokladnie wyrownane do strony
 * (funkcja nie zaokragla po cichu zakresu za wywolujacego — mapowania
 * fizyczne/wirtualne maja byc precyzyjne, w odroznieniu od API
 * przestrzeni adresowej procesu, ktore CELOWO zaokragla na granice
 * uzytkownika).
 */
static bool page_range_ok(uint64_t addr, size_t len)
{
	uint64_t start, end;

	if (!range_from_addr_len(addr, (uint64_t)len, ARCH_PAGE_SIZE,
	                         0, UINT64_MAX, false, &start, &end)) {
	    return false;
	}

	/* addr musi byc juz wyrownany, a len juz wielokrotnoscia strony —
	 * range_from_addr_len zaokraglilby inaczej w przeciwnym razie. */
	return start == addr && end == addr + (uint64_t)len;
}

bool mapping_map_range(mapping_space_t space,
	                   uint64_t virt,
	                   uint64_t phys,
	                   size_t len,
	                   uint64_t pte_flags)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len) || !page_range_ok(phys, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);
	size_t mapped = 0;

	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;
	    uint64_t p = phys + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_map_page_in(pml4, v, p, pte_flags)) {
	        for (size_t j = 0; j < mapped; j++) {
	            paging_unmap_page_in(pml4,
	                                 virt + (uint64_t)j * ARCH_PAGE_SIZE);
	        }

	        return false;
	    }

	    mapped++;
	}

	return true;
}

bool mapping_unmap_range(mapping_space_t space,
	                     uint64_t virt,
	                     size_t len)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

	for (size_t i = 0; i < pages; i++) {
	    paging_unmap_page_in(pml4, virt + (uint64_t)i * ARCH_PAGE_SIZE);
	}

	return true;
}

bool mapping_protect_range(mapping_space_t space,
	                       uint64_t virt,
	                       size_t len,
	                       uint64_t pte_flags)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(virt, len)) {
	    return false;
	}

	uint64_t pml4 = resolve_space(space);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

	/*
	 * P1 (paging/mapowania): walidacja transakcyjna — najpierw
	 * sprawdzamy, ze CALY zakres jest zmapowany, dopiero potem
	 * zmieniamy flagi jakiejkolwiek strony. Wczesniej brak jednej
	 * strony w POLOWIE zakresu przerywal petle po zmianie flag na
	 * prefiksie i zwracal false — wywolujacy dostawal informacje o
	 * niepowodzeniu, ale zakres zostawal w niespojnym stanie (czesc
	 * stron z nowymi uprawnieniami, czesc ze starymi).
	 */
	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_is_mapped_in(pml4, v)) {
	        return false;
	    }
	}

	for (size_t i = 0; i < pages; i++) {
	    uint64_t v = virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_set_flags_in(pml4, v, pte_flags)) {
	        /*
	         * Nie powinno wystapic po walidacji powyzej w obecnym,
	         * jednowatkowym (per-blokada) modelu wywolan. Gdyby jednak
	         * cos wspolbieznie zmienilo mapowanie miedzy pierwsza a
	         * druga petla, zatrzymujemy sie tutaj — to jest ostatnia
	         * linia obrony, NIE projektowana gwarancja atomowosci przy
	         * wspolbieznej modyfikacji. Pelne zabezpieczenie wymaga
	         * wspolnej blokady z VMA przy wyszukiwaniu i modyfikacji
	         * (patrz sekcja "P1 — semantyka mmap/VMA" w planie
	         * ulepszen: "wykonywac wyszukanie VMA pod ta sama blokada
	         * co modyfikacje").
	         */
	        return false;
	    }
	}

	return true;
}

uint64_t mapping_translate(mapping_space_t space, uint64_t virt)
{
	return paging_translate_in(resolve_space(space), virt);
}

bool mapping_is_mapped(mapping_space_t space, uint64_t virt)
{
	return paging_is_mapped_in(resolve_space(space), virt);
}

static void copy_page_by_phys(uint64_t dst_phys, uint64_t src_phys)
{
	uint64_t *dst = (uint64_t *)arch_phys_to_virt(dst_phys);
	const uint64_t *src = (const uint64_t *)arch_phys_to_virt(src_phys);

	for (size_t i = 0; i < ARCH_PAGE_SIZE / sizeof(uint64_t); i++) {
	    dst[i] = src[i];
	}

	__asm__ volatile("" ::: "memory");
}

bool mapping_copy_range(mapping_space_t dst,
	                    uint64_t dst_virt,
	                    mapping_space_t src,
	                    uint64_t src_virt,
	                    size_t len)
{
	if (len == 0) {
	    return true;
	}

	if (!page_range_ok(dst_virt, len) || !page_range_ok(src_virt, len)) {
	    return false;
	}

	uint64_t dst_pml4 = resolve_space(dst);
	uint64_t src_pml4 = resolve_space(src);

	size_t pages = (size_t)(len / ARCH_PAGE_SIZE);

	/*
	 * Poprawka błędu przy okazji (nie z numeracji P0/P1, ale ta sama
	 * klasa: cichy błąd na nieprawidłowym wejściu): `paging_translate_in`
	 * zwraca 0 dla niezmapowanego adresu (patrz jego własny komentarz —
	 * "physical address 0 is then indistinguishable"), a NIE
	 * `UINT64_MAX`. Poprzedni warunek `dp == UINT64_MAX || sp ==
	 * UINT64_MAX` w praktyce nigdy nie wykrywał braku mapowania — funkcja
	 * mogła cicho skopiować zawartość spod fizycznego adresu 0 zamiast
	 * odmówić. Używamy `paging_is_mapped_in()`, jedynego jednoznacznego
	 * sposobu sprawdzenia (zgodnie z dokumentacją `paging_translate_in`).
	 */
	for (size_t i = 0; i < pages; i++) {
	    uint64_t dv = dst_virt + (uint64_t)i * ARCH_PAGE_SIZE;
	    uint64_t sv = src_virt + (uint64_t)i * ARCH_PAGE_SIZE;

	    if (!paging_is_mapped_in(dst_pml4, dv) ||
	        !paging_is_mapped_in(src_pml4, sv)) {
	        return false;
	    }

	    uint64_t dp = paging_translate_in(dst_pml4, dv);
	    uint64_t sp = paging_translate_in(src_pml4, sv);

	    copy_page_by_phys(dp, sp);
	}

	return true;
}