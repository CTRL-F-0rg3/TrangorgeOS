#include "buddy.h"
#include "../physical/pmm.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"

extern void kprintf(const char *fmt, ...);

#define BUDDY_STATE_FREE 0
#define BUDDY_STATE_USED 1
#define BUDDY_STATE_TAIL 2

static uint64_t buddy_base = 0;
static size_t buddy_pages = 0, buddy_max_order = 0;

static uint8_t *buddy_order = NULL, *buddy_state = NULL;
static int32_t *buddy_next = NULL, *buddy_heads = NULL;

static buddy_map_cb buddy_map = NULL;
static buddy_unmap_cb buddy_unmap = NULL;

static bool buddy_initialized = false;

static size_t buddy_used_bytes = 0, buddy_free_bytes = 0;

static inline uint64_t order_size(size_t o) { return ARCH_PAGE_SIZE << o; }
static size_t page_of(uint64_t va) { return (size_t)((va - buddy_base) / ARCH_PAGE_SIZE); }
static uint64_t va_of(size_t page) { return buddy_base + (uint64_t)page * ARCH_PAGE_SIZE; }
static size_t size_to_order(size_t size) {
	size_t o = 0, s = ARCH_PAGE_SIZE;
	while (s < size && o < buddy_max_order) {
	    s <<= 1;
	    o++;
	}
	return o;
}
static void fl_push(size_t o, size_t idx) {
	buddy_next[idx] = buddy_heads[o];
	buddy_heads[o] = (int32_t)idx;
}
static size_t fl_pop(size_t o) {
	int32_t i = buddy_heads[o];
	if (i < 0) return (size_t)-1;
	buddy_heads[o] = buddy_next[i];
	return (size_t)i;
}
static void fl_remove(size_t o, size_t idx) {
	int32_t *p = &buddy_heads[o];
	while (*p >= 0) {
	    if ((size_t)*p == idx) {
	        *p = buddy_next[*p];
	        return;
	    }
	    p = &buddy_next[*p];
	}
}
bool buddy_init(uint64_t base, size_t size, buddy_map_cb map_cb, buddy_unmap_cb unmap_cb) {
	if (buddy_initialized) return true;
	if (map_cb == NULL || unmap_cb == NULL) return false;
	if (size == 0 || (size & (size - 1)) != 0) return false;
	if (size < ARCH_PAGE_SIZE) return false;
	if (!arch_is_page_aligned(base)) return false;
	if ((base & (size - 1)) != 0) return false;
	buddy_base = base;
	buddy_pages = size / ARCH_PAGE_SIZE;
	buddy_max_order = 0;
	while (order_size(buddy_max_order) < size) buddy_max_order++;
	size_t
		off_order = 0,
		off_state = buddy_pages,
		off_next = (off_state + buddy_pages + 7) & ~(size_t)7,
		off_heads = (off_next + buddy_pages * 4 + 7) & ~(size_t)7,
		meta_bytes = off_heads + (buddy_max_order + 1) * 4;
	uint64_t meta_phys = 0;
	if (!pmm_alloc_bytes(meta_bytes, &meta_phys)) return false;
	uint8_t *meta = (uint8_t *)arch_phys_to_virt(meta_phys);
	buddy_order = meta + off_order;
	buddy_state = meta + off_state;
	buddy_next = (int32_t *)(meta + off_next);
	buddy_heads = (int32_t *)(meta + off_heads);
	for (size_t i = 0; i < buddy_pages; i++) {
	    buddy_order[i] = 0;
	    buddy_state[i] = BUDDY_STATE_TAIL;
	}
	for (size_t o = 0; o <= buddy_max_order; o++) buddy_heads[o] = -1;
	buddy_state[0] = BUDDY_STATE_FREE;
	buddy_order[0] = (uint8_t)buddy_max_order;
	fl_push(buddy_max_order, 0);
	buddy_map = map_cb;
	buddy_unmap = unmap_cb;
	buddy_free_bytes = size;
	buddy_used_bytes = 0;
	buddy_initialized = true;
	return true;
}
bool buddy_ready(void) { return buddy_initialized; }
void *buddy_alloc(size_t size) {
	if (!buddy_initialized || size == 0) return NULL;
	size_t o = size_to_order(size);
	if (order_size(o) < size || o > buddy_max_order) return NULL;
	size_t cur = o;
	while (buddy_heads[cur] < 0 && cur < buddy_max_order) cur++;
	if (buddy_heads[cur] < 0) return NULL;
	size_t idx = fl_pop(cur);
	while (cur > o) {
	    cur--;
	    size_t half_pages = (size_t)1 << cur, buddy_idx = idx + half_pages;
	    buddy_state[buddy_idx] = BUDDY_STATE_FREE;
	    buddy_order[buddy_idx] = (uint8_t)cur;
	    fl_push(cur, buddy_idx);
	}
	buddy_order[idx] = (uint8_t)o;
	buddy_state[idx] = BUDDY_STATE_USED;
	size_t bytes = order_size(o);
	if (!buddy_map(va_of(idx), bytes)) {
	    buddy_state[idx] = BUDDY_STATE_FREE;
	    fl_push(o, idx);
	    return NULL;
	}
	buddy_used_bytes += bytes;
	buddy_free_bytes -= bytes;
	return (void *)(uintptr_t)va_of(idx);
}

void *buddy_alloc_aligned(size_t size, size_t align) {
	if (!buddy_initialized || size == 0) return NULL;
	if (!size_is_pow2(align)) return NULL;
	if (align <= ARCH_PAGE_SIZE) return buddy_alloc(size);
	size_t need = size;
	if (align > need) need = align;
	size_t p;
	if (!size_round_up_pow2(need, &p)) return NULL;
	if (p < ARCH_PAGE_SIZE) p = ARCH_PAGE_SIZE;
	return buddy_alloc(p);
}

void buddy_free(void *ptr) {
	if (!buddy_initialized || ptr == NULL) return;
	uint64_t va = (uint64_t)(uintptr_t)ptr;
	if (va < buddy_base || va >= buddy_base + (uint64_t)buddy_pages * ARCH_PAGE_SIZE) return;
	size_t idx = page_of(va);
	if (buddy_state[idx] != BUDDY_STATE_USED) return;
	size_t o = buddy_order[idx], bytes = order_size(o);
	buddy_unmap(va, bytes);
	buddy_state[idx] = BUDDY_STATE_FREE;
	buddy_order[idx] = (uint8_t)o;
	size_t cur = o;
	while (cur < buddy_max_order) {
	    size_t buddy_idx = idx ^ ((size_t)1 << cur);
	    if (buddy_state[buddy_idx] != BUDDY_STATE_FREE ||
	        buddy_order[buddy_idx] != cur) {
	        break;
	    }
	    fl_remove(cur, buddy_idx);
	    buddy_state[buddy_idx] = BUDDY_STATE_TAIL;
	    if (buddy_idx < idx) idx = buddy_idx;
	    cur++;
	    buddy_order[idx] = (uint8_t)cur;
	}
	fl_push(cur, idx);
	buddy_used_bytes -= bytes;
	buddy_free_bytes += bytes;
}
size_t buddy_block_size(void *ptr) {
	if (!buddy_initialized || ptr == NULL) return 0;
	uint64_t va = (uint64_t)(uintptr_t)ptr;
	if (va < buddy_base || va >= buddy_base + (uint64_t)buddy_pages * ARCH_PAGE_SIZE) return 0;
	size_t idx = page_of(va);
	if (buddy_state[idx] != BUDDY_STATE_USED) return 0;
	return (size_t)order_size(buddy_order[idx]);
}
size_t buddy_stat_used_bytes(void) { return buddy_used_bytes; }
size_t buddy_stat_free_bytes(void) { return buddy_free_bytes; }
void buddy_dump(void) {
	if (!buddy_initialized) {
	    kprintf("BUDDY: not initialized\n");
	    return;
	}
	kprintf("BUDDY:\n");
	kprintf("  base: 0x%llx\n", (unsigned long long)buddy_base);
	kprintf("  max order: %llu\n", (unsigned long long)buddy_max_order);
	kprintf("  used: %llu KiB\n", (unsigned long long)(buddy_used_bytes >> 10));
	kprintf("  free: %llu KiB\n", (unsigned long long)(buddy_free_bytes >> 10));
}
