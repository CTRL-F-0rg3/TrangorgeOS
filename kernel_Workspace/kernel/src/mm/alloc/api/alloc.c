#include "alloc.h"
#include "../heap/heap.h"
#include "../virtual/mapping.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"
#include "../../../types.h"

extern void kprintf(const char *fmt, ...);

static void k_memset(const void *dst, const uint8 value, const size_t n) {
	uint8_t *p = (uint8_t *)dst;
	for (size_t i = 0; i < n; i++) p[i] = value;
}

static void k_memcpy(const void *dst, const void *src, const size_t n) {
	uint8_t *d = (uint8_t *)dst;
	const uint8_t *s = (const uint8_t *)src;
	for (size_t i = 0; i < n; i++) d[i] = s[i];
}

static size_t k_strlen(const char *s) {
	size_t n = 0;
	while (s[n] != '\0') n++;
	return n;
}

static size_t k_usable_size(void *ptr) {
#ifdef ALLOC_DEBUG
	size_t s = dbg_usable_size(ptr);
	if (s != 0) return s;
#endif
	return heap_usable_size(ptr);
}

void *kmalloc(const size_t size) {
	if (size == 0) return NULL;
#ifdef ALLOC_DEBUG
	return dbg_alloc(size);
#else
	return heap_alloc(size);
#endif
}

void kfree(void *ptr) {
	if (ptr == NULL) return;
#ifdef ALLOC_DEBUG
	dbg_free(ptr);
#else
	heap_free(ptr);
#endif
}

void *kzalloc(const size_t size) {
	void *ptr = kmalloc(size);
	if (ptr == NULL) return NULL;
	k_memset(ptr, 0, size);
	return ptr;
}

void *kcalloc(const size_t count, const size_t size) {
	if (count == 0 || size == 0) return NULL;
	if (count > SIZE_MAX / size) return NULL;
	return kzalloc(count * size);
}

void *kmalloc_aligned(const size_t size, const size_t align) {
	if (size == 0) return NULL;

	if (!size_is_pow2(align)) return NULL;
	return heap_alloc_aligned(size, align);
}


size_t kmalloc_usable_size(void *ptr) { return k_usable_size(ptr); }

void *krealloc(void *ptr, const size_t new_size) {
	if (ptr == NULL) return kmalloc(new_size);
	if (new_size == 0) {
		kfree(ptr);
		return NULL;
	}
	const size_t old_size = k_usable_size(ptr);
	if (old_size == 0) return NULL;
	if (old_size >= new_size) return ptr;
	void *new_ptr = kmalloc(new_size);
	if (new_ptr == NULL) return NULL;
	k_memcpy(new_ptr, ptr, old_size);
	kfree(ptr);
	return new_ptr;
}

void *kalloc_pages(const size_t pages) {
	if (pages == 0) return NULL;
	if (pages > SIZE_MAX / ARCH_PAGE_SIZE) return NULL;
	return heap_alloc(pages * ARCH_PAGE_SIZE);
}

void kfree_pages(void *ptr, const size_t pages) {
	if (ptr == NULL) return;
	size_t requested_bytes = 0;
	const size_t usable = heap_usable_size(ptr);
	if (!kfree_pages_validate(pages, ARCH_PAGE_SIZE, usable, &requested_bytes)) {
		kprintf("kfree_pages: nieprawidlowy parametr pages=%zu (pojemnosc bloku=%zu B) dla ptr=%p — odmowa zwolnienia\n", pages, usable, ptr);
		return;
	}
	heap_free(ptr);
}

char *kstrdup(const char *s) {
	if (s == NULL) return NULL;
	const size_t n = k_strlen(s) + 1;
	char *ptr = (char *)kmalloc(n);
	if (ptr == NULL) return NULL;
	k_memcpy(ptr, s, n);
	return ptr;
}

uint64_t kvirt_to_phys(void *ptr) {
	if (ptr == NULL) return UINT64_MAX;
	const uint64_t va = (uint64_t)(uintptr_t)ptr,
	phys = mapping_translate(MAPPING_KERNEL, va);
	if (phys != UINT64_MAX) return phys;
	return arch_virt_to_phys(ptr);
}

void kalloc_dump(void) {
#ifdef ALLOC_DEBUG
	mm_debug_dump();
#else
	heap_dump();
#endif
}
