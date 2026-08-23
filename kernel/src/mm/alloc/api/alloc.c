#include "alloc.h"
#include "../heap/heap.h"
// #include "../debug/alloc_debug.h"
#include "../virtual/mapping.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"

extern void kprintf(const char *fmt, ...);

static void k_memset(void *dst, uint8_t value, size_t n)
{
    uint8_t *p = (uint8_t *)dst;

    for (size_t i = 0; i < n; i++) {
        p[i] = value;
    }
}

static void k_memcpy(void *dst, const void *src, size_t n)
{
    uint8_t *d = (uint8_t *)dst;
    const uint8_t *s = (const uint8_t *)src;

    for (size_t i = 0; i < n; i++) {
        d[i] = s[i];
    }
}

static size_t k_strlen(const char *s)
{
    size_t n = 0;

    while (s[n] != '\0') {
        n++;
    }

    return n;
}

static size_t k_usable_size(void *ptr)
{
#ifdef ALLOC_DEBUG
    size_t s = dbg_usable_size(ptr);

    if (s != 0) {
        return s;
    }
#endif

    return heap_usable_size(ptr);
}

void *kmalloc(size_t size)
{
    if (size == 0) {
        return NULL;
    }

#ifdef ALLOC_DEBUG
    return dbg_alloc(size);
#else
    return heap_alloc(size);
#endif
}

void kfree(void *ptr)
{
    if (ptr == NULL) {
        return;
    }

#ifdef ALLOC_DEBUG
    dbg_free(ptr);
#else
    heap_free(ptr);
#endif
}

void *kzalloc(size_t size)
{
    void *ptr = kmalloc(size);

    if (ptr == NULL) {
        return NULL;
    }

    k_memset(ptr, 0, size);

    return ptr;
}

void *kcalloc(size_t count, size_t size)
{
    if (count == 0 || size == 0) {
        return NULL;
    }

    if (count > SIZE_MAX / size) {
        return NULL;
    }

    return kzalloc(count * size);
}

void *kmalloc_aligned(size_t size, size_t align)
{
    if (size == 0) {
        return NULL;
    }

    /*
     * P1.1: kontrakt `align` musi być potęgą dwójki (jak
     * `aligned_alloc`/`posix_memalign` z libc). Wcześniej dowolna
     * wartość była cicho akceptowana i mogła dawać blok, który NIE
     * spełnia żądanego wyrównania (patrz komentarz w
     * buddy_alloc_aligned()). Walidacja tutaj jest tylko obroną w głąb —
     * heap_alloc_aligned()/buddy_alloc_aligned() i tak to sprawdzają.
     */
    if (!size_is_pow2(align)) {
        return NULL;
    }

    return heap_alloc_aligned(size, align);
}

/*
 * Zwraca pojemność bloku alokatora stojącego za `ptr`.
 *
 * UWAGA (P1.3, kontrakt dokumentowany zamiast domyślany): w budowie z
 * ALLOC_DEBUG zwracana wartość jest DOKŁADNYM rozmiarem żądanym w
 * kmalloc()/kzalloc()/kcalloc() (śledzonym w nagłówku debug — patrz
 * alloc_debug.c). W budowie RELEASE (bez ALLOC_DEBUG) zwracana wartość
 * to POJEMNOŚĆ bloku leżącego pod spodem (buddy: zaokrąglona w górę do
 * potęgi dwójki stron; slab: rozmiar klasy) — może więc być WIĘKSZA niż
 * to, co faktycznie przekazano do kmalloc(). To jest bezpieczne do
 * odczytu/zapisu (blok naprawdę ma tyle miejsca), ale nie należy tej
 * wartości traktować jako "dokładnego rozmiaru ostatniej alokacji" przy
 * podejmowaniu decyzji logicznych (np. serializacji) w budowie release.
 * Pełne ujednolicenie (nagłówek z dokładnym rozmiarem w KAŻDEJ budowie)
 * wymaga osobnej, większej zmiany obejmującej kfree()/heap_free() i jest
 * świadomie odłożone poza zakres tej sesji ze względu na promień
 * rażenia (dotyka też kmalloc_aligned/kalloc_pages, które mają inne
 * reguły wyrównania i własne funkcje zwalniające).
 */
size_t kmalloc_usable_size(void *ptr)
{
    return k_usable_size(ptr);
}

void *krealloc(void *ptr, size_t new_size)
{
    if (ptr == NULL) {
        return kmalloc(new_size);
    }

    if (new_size == 0) {
        kfree(ptr);
        return NULL;
    }

    size_t old_size = k_usable_size(ptr);

    if (old_size == 0) {
        return NULL;
    }

    if (old_size >= new_size) {
        return ptr;
    }

    void *new_ptr = kmalloc(new_size);

    if (new_ptr == NULL) {
        return NULL;
    }

    k_memcpy(new_ptr, ptr, old_size);

    kfree(ptr);

    return new_ptr;
}

void *kalloc_pages(size_t pages)
{
    if (pages == 0) {
        return NULL;
    }

    if (pages > SIZE_MAX / ARCH_PAGE_SIZE) {
        return NULL;
    }

    return heap_alloc(pages * ARCH_PAGE_SIZE);
}

void kfree_pages(void *ptr, size_t pages)
{
    if (ptr == NULL) {
        return;
    }

    /*
     * P1.2: `pages` był wcześniej całkowicie ignorowany (`(void)pages`),
     * więc np. `kfree_pages(ptr, 999999)` po `kalloc_pages(1)` cicho
     * zwalniało prawdziwy (mały) blok, dając wywołującemu fałszywe
     * poczucie bezpieczeństwa co do tego, ile pamięci właśnie oddał.
     *
     * Nie ma tu osobnego alokatora stron ze śladem dokładnej długości
     * (kalloc_pages() świadomie korzysta wprost z heap_alloc(), z
     * pominięciem ALLOC_DEBUG — tak samo jak tutaj używamy heap_free()),
     * więc pełna walidacja "dokładnie tylu stron ile zażądano przy
     * alokacji" nie jest możliwa bez dodatkowych metadanych. Zamiast
     * tego wykonujemy sprawdzenie spójności: żądana liczba stron nie
     * może przekraczać rzeczywistej pojemności bloku pod `ptr` — to
     * wyłapuje ewentualne błędy wywołującego (zła stała, przekazany zły
     * wskaźnik, pomylona jednostka bajty/strony), zamiast je ukrywać.
     */
    size_t requested_bytes = 0;
    size_t usable = heap_usable_size(ptr);

    if (!kfree_pages_validate(pages, ARCH_PAGE_SIZE, usable, &requested_bytes)) {
        kprintf("kfree_pages: nieprawidlowy parametr pages=%zu (pojemnosc "
                "bloku=%zu B) dla ptr=%p — odmowa zwolnienia\n",
                pages, usable, ptr);
        return;
    }

    heap_free(ptr);
}

char *kstrdup(const char *s)
{
    if (s == NULL) {
        return NULL;
    }

    size_t n = k_strlen(s) + 1;

    char *ptr = (char *)kmalloc(n);

    if (ptr == NULL) {
        return NULL;
    }

    k_memcpy(ptr, s, n);

    return ptr;
}

uint64_t kvirt_to_phys(void *ptr)
{
    if (ptr == NULL) {
        return UINT64_MAX;
    }

    uint64_t va = (uint64_t)(uintptr_t)ptr;

    uint64_t phys = mapping_translate(MAPPING_KERNEL, va);

    if (phys != UINT64_MAX) {
        return phys;
    }

    return arch_virt_to_phys(ptr);
}

void kalloc_dump(void)
{
#ifdef ALLOC_DEBUG
    mm_debug_dump();
#else
    heap_dump();
#endif
}
