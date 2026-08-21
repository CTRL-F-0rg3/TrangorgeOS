#include "mmap.h"
#include "../protection/permissions.h"

uint64_t mmap(proc_aspace_t *pa,
              uint64_t addr,
              size_t len,
              uint32_t prot,
              uint32_t flags)
{
    if (pa == NULL || len == 0) {
        return 0;
    }

    prot = perm_sanitize(prot);

    if (flags & MAP_FIXED) {
        return aspace_map_at(pa, addr, len, prot);
    }

    return aspace_map_anon(pa, addr, len, prot);
}

bool munmap(proc_aspace_t *pa, uint64_t addr, size_t len)
{
    return aspace_unmap(pa, addr, len);
}

/*
 * P1 (mmap/VMA): wcześniej ta funkcja wołała `aspace_vma_find(pa, addr)`
 * BEZ TRZYMANIA żadnej blokady, po czym czytała `v->prot` w
 * `perm_mprotect_allowed(v->prot, prot)` — czysty use-after-free, gdyby
 * inny rdzeń w tym samym momencie zwolnił tę VMA przez `munmap()` (VMA
 * jest alokowana przez `heap_alloc()` i zwalniana przez `heap_free()`
 * wewnątrz `aspace_unmap()`, więc odczyt po zwolnieniu czytałby albo
 * odzyskaną pamięć, albo — po ponownym użyciu tego bloku przez inną
 * alokację — całkowicie niepowiązane dane zinterpretowane jako `prot`).
 *
 * `aspace_protect_checked()` wykonuje wyszukanie VMA, sprawdzenie
 * `perm_mprotect_allowed()` i samą zmianę uprawnień pod JEDNYM trzymaniem
 * wewnętrznej blokady przestrzeni adresowej — eliminuje to okno TOCTOU.
 *
 * Sprawdzenie (`checked_prot`) celowo dostaje SUROWE `prot` — tak jak w
 * oryginalnym kodzie — a NIE `perm_sanitize(prot)`: `perm_mprotect_allowed`
 * odrzuca jawne żądanie PROT_WRITE|PROT_EXEC (W^X) na podstawie tego, o co
 * POPROSZONO. Gdyby sprawdzać już-oczyszczoną wartość, `perm_sanitize()`
 * zdążyłby wcześniej po cichu zdjąć PROT_EXEC, i jawne żądanie W^X nigdy
 * nie zostałoby odrzucone — tylko po cichu obniżone. Do zastosowania
 * (`apply_prot`) idzie już `perm_sanitize(prot)`, dokładnie jak wcześniej.
 */
bool mprotect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
{
    return aspace_protect_checked(pa, addr, len,
                                  prot, perm_sanitize(prot),
                                  perm_mprotect_allowed);
}