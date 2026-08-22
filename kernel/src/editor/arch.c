/* arch.c — silne symbole pomocnicze edytora (kernel/src/editor/editor.c).
 *
 * editor.c deklaruje `extern void *arch_phys_to_virt(uint64_t phys)`, a
 * mm/arch/x86_64/memory.h podaje tylko `static inline`. Dopóki edytor nie
 * był wywoływany, linker nie dokładał editor.o; po podpięciu komendy
 * `edit` musi istnieć silny (extern) symbol — dlatego definiujemy go tutaj.
 *
 * Uwaga: celowo nie włączamy memory.h, bo to wywołałoby błąd redefinicji
 * względem `static inline` z nagłówka — trzymamy spójną stałą direct mapy.
 */

#include <stdint.h>

#define ARCH_DIRECT_MAP_BASE 0xFFFF888000000000UL

void *arch_phys_to_virt(uint64_t phys)
{
    return (void *)(uintptr_t)(phys + ARCH_DIRECT_MAP_BASE);
}