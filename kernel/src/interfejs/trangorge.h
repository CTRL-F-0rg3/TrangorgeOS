#ifndef TRANGORGE_H
#define TRANGORGE_H

/*
 * interfejs/trangorge.h — główny nagłówek zbiorczy (umbrella header)
 * TrangorgeOS — dołącza wszystkie pozostałe nagłówki jądra.
 * Ścieżki względne wobec kernel/src/interfejs/.
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




/*  memory management functions */

/* page management */

/*  paging functions */

/*ipc*/

/*devices   */

/*shelduler*/




#endif // TRANGORGE_H