#ifndef MM_PROCESS_ADDRESS_SPACE_H
#define MM_PROCESS_ADDRESS_SPACE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "../paging/paging.h"

#define VMA_FLAG_ANON    (1u << 0)
#define VMA_FLAG_PRIVATE (1u << 1)
#define VMA_FLAG_SHARED  (1u << 2)
#define VMA_FLAG_GUARD   (1u << 3)

typedef struct vma {
    uint64_t start;
    uint64_t end;
    uint32_t prot;
    uint32_t flags;
    struct vma *next;
} vma_t;

typedef struct proc_aspace {
    address_space_t *as;
    vma_t *vmas;
    uint64_t brk_base;
    uint64_t brk;
    uint64_t brk_max;
} proc_aspace_t;

bool aspace_subsystem_init(void);

proc_aspace_t *aspace_create(void);
void aspace_destroy(proc_aspace_t *pa);

address_space_t *aspace_paging_handle(proc_aspace_t *pa);

uint64_t aspace_map_anon(proc_aspace_t *pa, uint64_t hint, size_t len, uint32_t prot);
uint64_t aspace_map_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);
uint64_t aspace_reserve_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t flags);
bool aspace_unmap(proc_aspace_t *pa, uint64_t addr, size_t len);
bool aspace_protect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);

/*
 * P1 (mmap/VMA — "mprotect() wyszukuje VMA przed wejściem w blokadę"):
 * wariant `aspace_protect()`, który wykonuje wyszukanie VMA, sprawdzenie
 * dozwolonej zmiany uprawnień i samą zmianę — WSZYSTKO pod JEDNYM
 * trzymaniem wewnętrznej blokady przestrzeni adresowej. Bez tego
 * wywołujący (patrz dawny `mprotect()` w process/mmap.c) musiałby wywołać
 * `aspace_vma_find()` osobno, bez blokady, a potem czytać `v->prot` z
 * NIEZABLOKOWANEGO wskaźnika — inny rdzeń mógłby w tym czasie zwolnić tę
 * samą VMA przez `munmap()`, zamieniając odczyt `v->prot` w
 * use-after-free.
 *
 * `checked_prot` i `apply_prot` są celowo ROZDZIELONE: `allowed(old_prot,
 * checked_prot)` decyduje na podstawie tego, o co POPROSZONO (np.
 * `perm_mprotect_allowed()` odrzuca jawne żądanie PROT_WRITE|PROT_EXEC —
 * W^X — zamiast po cichu je obniżać), a `apply_prot` to wartość faktycznie
 * zapisywana do VMA i tablic stron (np. już przepuszczona przez
 * `perm_sanitize()`). Scalenie tych dwóch w jeden parametr sprawiałoby,
 * że sprawdzenie odbywałoby się na już-oczyszczonej wartości i nigdy nie
 * wykrywałoby jawnego żądania W^X.
 *
 * `allowed` może być NULL — wtedy każda zmiana jest dozwolona (taka jak
 * `aspace_protect()`, która wewnętrznie woła tę funkcję z `allowed=NULL`
 * i `checked_prot == apply_prot`).
 */
bool aspace_protect_checked(proc_aspace_t *pa, uint64_t addr, size_t len,
                            uint32_t checked_prot,
                            uint32_t apply_prot,
                            bool (*allowed)(uint32_t old_prot,
                                            uint32_t checked_prot));

/*
 * UWAGA (bezpieczeństwo współbieżności): zwrócony wskaźnik jest ważny
 * TYLKO tak długo, jak wywołujący trzyma wewnętrzną blokadę tej
 * przestrzeni adresowej — funkcja sama NIE blokuje. Odczyt jakiegokolwiek
 * pola spod zwróconego wskaźnika (np. `v->prot`) PO zwolnieniu blokady
 * (albo bez jej wcześniejszego wzięcia) jest niezdefiniowane — inny
 * rdzeń może w tym czasie zwolnić tę VMA przez `aspace_unmap()`. Do
 * odczytu/zmiany uprawnień z zewnątrz modułu używaj
 * `aspace_protect_checked()`, która sama zarządza blokadą.
 */
vma_t *aspace_vma_find(proc_aspace_t *pa, uint64_t addr);
uint64_t aspace_stack_base(void);

uint64_t aspace_brk(proc_aspace_t *pa, uint64_t new_brk);

#endif