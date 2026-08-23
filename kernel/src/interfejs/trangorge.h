#ifndef TRANGORGE_H
#define TRANGORGE_H

/*
 * interfejs/trangorge.h — główny nagłówek zbiorczy (umbrella header)
 * TrangorgeOS — dołącza wszystkie pozostałe nagłówki jądra.
 * Ścieżki względne wobec kernel/src/interfejs/.
 *
 * PEŁNA LISTA TEGO, CO UDOSTĘPNIAJĄ TE NAGŁÓWKI (typy, funkcje, stałe,
 * sekcja po sekcji) znajduje się w pliku:  interfejs/trangorge_api.txt
 *
 * Skrótowy przegląd modułów:
 *   types.h        — aliasy szerokości całkowitych (int8..uint64, uint)
 *   safe.h/policy.h— bezpieczeństwo i polityki interfejsu
 *   battery/       — status baterii (init/operation/backend)
 *   bluetooth/     — HCI, ACL tx/rx, model zdarzeń
 *   camera/        — przechwytywanie klatek, formaty RGB888/YUYV
 *   core-lang/     — język wewnętrzny: lexer/parser/sema/codegen/VM,
 *                    arytmetyka 4..512-bit, natywny JIT, bridge ringów
 *   displayport/   — AUX/DPCD, EDID, trening linku, framebuffer DP
 *   editor/        — edytor tekstu + mysz
 *   hdmi/          — tryby ekranu, kolejka transferów (blit/fill/flip)
 *   linuxcom/      — warstwa kompatybilności "lc_*" (wątki, VFS, sockety,
 *                    blokady, workqueue/timery, urządzenia)
 *   mm/alloc/      — kmalloc/kfree (api), debug+leaki+statystyki,
 *                    heap/slab/buddy, ramki fizyczne+pmm+strefy DMA32/NORMAL,
 *                    contiguous/dma-coherent, mapping/page/vmm
 *   mm/arch/...    — pamięć/paging/TLB per architektura (warunkowe)
 *   mm/cache/      — obiektowe cache (kcache), per-CPU
 *   mm/core/       — adresy, zakresy, regiony, sizeutil, smp ticket lock
 *   mm/paging/     — aspace wrapper, PML helpery, batch TLB
 *   mm/process/    — VMA, mmap/munmap/mprotect, brk
 *   mm/protection/ — guard page, SMEP/SMAP, sanity uprawnień (W^X)
 */

/* === typy podstawowe === */
#include "../types.h"
#include "safe.h"
#include "policy.h"
/* === battery === */
#include "../battery/battery.h"
#include "../battery/init.h"
#include "../battery/operation.h"

/* === bluetooth === */
#include "../bluetooth/bt.h"
#include "../bluetooth/hci.h"
#include "../bluetooth/init.h"
#include "../bluetooth/operation.h"

/* === camera === */
#include "../camera/camera.h"
#include "../camera/init.h"
#include "../camera/operation.h"

/* === core-lang === */
#include "../core-lang/ast.h"
#include "../core-lang/bc.h"
#include "../core-lang/bridge.h"
#include "../core-lang/lexer.h"
#include "../core-lang/loader.h"
#include "../core-lang/native.h"
#include "../core-lang/parser.h"
#include "../core-lang/sema.h"
#include "../core-lang/tokens.h"
#include "../core-lang/vm.h"

/* === displayport === */
#include "../displayport/aux.h"
#include "../displayport/dp.h"
#include "../displayport/edid.h"
#include "../displayport/init.h"
#include "../displayport/link.h"
#include "../displayport/operation.h"

/* === editor === */
#include "../editor/editor.h"
#include "../editor/mouse.h"

/* === hdmi === */
#include "../hdmi/hdmi.h"
#include "../hdmi/init.h"
#include "../hdmi/mode.h"
#include "../hdmi/operation.h"

/* === linuxcom === */
#include "../linuxcom/compat.h"

/* === mm: alloc/api === */
#include "../mm/alloc/api/alloc.h"

/* === mm: alloc/debug === */
#include "../mm/alloc/debug/alloc_debug.h"
#include "../mm/alloc/debug/leak.h"
#include "../mm/alloc/debug/stats.h"

/* === mm: alloc/heap === */
#include "../mm/alloc/heap/buddy.h"
#include "../mm/alloc/heap/heap.h"
#include "../mm/alloc/heap/slab.h"

/* === mm: alloc/physical === */
#include "../mm/alloc/physical/bitmap.h"
#include "../mm/alloc/physical/frame.h"
#include "../mm/alloc/physical/pmm.h"

/* === mm: alloc/special === */
#include "../mm/alloc/special/contiguous.h"
#include "../mm/alloc/special/dma.h"

/* === mm: alloc/virtual === */
#include "../mm/alloc/virtual/mapping.h"
#include "../mm/alloc/virtual/page.h"
#include "../mm/alloc/virtual/vmm.h"
/* === mm: arch (zależnie od architektury) === */
#if defined(__x86_64__)
#include "../mm/arch/x86_64/memory.h"
#include "../mm/arch/x86_64/paging.h"
#include "../mm/arch/x86_64/tlb.h"
#include "../mm/arch/x86_64/tlb_asm.h"
#elif defined(__aarch64__)
#include "../mm/arch/aarch64/memory.h"
#include "../mm/arch/aarch64/paging.h"
#include "../mm/arch/aarch64/tlb.h"
#endif

/* === mm: cache === */
#include "../mm/cache/cache.h"
#include "../mm/cache/object_cashe.h"
#include "../mm/cache/per_cpu.h"

/* === mm: core === */
#include "../mm/core/address.h"
#include "../mm/core/mm.h"
#include "../mm/core/range.h"
#include "../mm/core/region.h"
#include "../mm/core/sizeutil.h"
#include "../mm/core/smp_lock.h"

/* === mm: paging === */
#include "../mm/paging/paging.h"
#include "../mm/paging/pml.h"
#include "../mm/paging/tlb.h"

/* === mm: process === */
#include "../mm/process/address_space.h"
#include "../mm/process/mmap.h"

/* === mm: protection === */
#include "../mm/protection/guard.h"
#include "../mm/protection/isolation.h"
#include "../mm/protection/permissions.h"

#endif // TRANGORGE_H