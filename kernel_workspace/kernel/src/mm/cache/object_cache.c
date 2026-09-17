#include "object_cashe.h"
#include "../alloc/heap/buddy.h"
#include "../alloc/heap/heap.h"
#include "../arch/x86_64/memory.h"
#include "cache.h"
static size_t cache_lock_depth = 0;
static uint64_t cache_lock_flags = 0;

static void cache_lock(void)
{
	uint64_t flags;

	__asm__ volatile(
	    "pushfq\n"
	    "popq %0\n"
	    "cli"
	    : "=r"(flags)
	    :
	    : "memory"
	);

	if (cache_lock_depth == 0) {
	    cache_lock_flags = flags;
	}

	cache_lock_depth++;
}

static void cache_unlock(void)
{
	if (cache_lock_depth == 0) {
	    return;
	}

	cache_lock_depth--;

	if (cache_lock_depth == 0) {
	    uint64_t flags = cache_lock_flags;

	    __asm__ volatile(
	        "pushq %0\n"
	        "popfq"
	        :
	        : "r"(flags)
	        : "memory"
	    );
	}
}

static void cache_memset(void *dst, uint8_t value, size_t n)
{
	uint8_t *p = (uint8_t *)dst;

	for (size_t i = 0; i < n; i++) {
	    p[i] = value;
	}
}

static size_t cache_strlen(const char *s)
{
	size_t n = 0;

	while (s[n] != '\0') {
	    n++;
	}

	return n;
}

static bool kcache_grow(kcache_t *c)
{
	void *va = buddy_alloc(c->chunk_size);

	if (va == NULL) {
	    return false;
	}

	kcache_chunk_t *ch =
	    (kcache_chunk_t *)heap_alloc(sizeof(kcache_chunk_t));

	if (ch == NULL) {
	    buddy_free(va);
	    return false;
	}

	ch->va = (uint64_t)(uintptr_t)va;
	ch->next = c->chunks;
	c->chunks = ch;

	size_t n = c->chunk_size / c->object_size;

	for (size_t i = 0; i < n; i++) {
	    uint8_t *obj =
	        (uint8_t *)va + (uint64_t)i * c->object_size;

	    if (c->ctor != NULL) {
	        c->ctor(obj);
	    }

	    *(void **)obj = c->free_head;
	    c->free_head = obj;
	}

	c->free_count += n;
	c->total_objects += n;

	return true;
}

kcache_t *kcache_create(const char *name,
	                    size_t object_size,
	                    kcache_ctor_t ctor)
{
	if (object_size == 0) {
	    return NULL;
	}

	if (object_size < sizeof(void *)) {
	    object_size = sizeof(void *);
	}

	object_size = (object_size + 15) & ~(size_t)15;

	size_t chunk = object_size * 32;

	if (chunk < 4096) {
	    chunk = 4096;
	}

	size_t p = 4096;

	while (p < chunk) {
	    p <<= 1;
	}

	chunk = p;

	kcache_t *c = (kcache_t *)heap_zalloc(sizeof(kcache_t));

	if (c == NULL) {
	    return NULL;
	}

	size_t n = cache_strlen(name);

	if (n >= KCACHE_NAME_MAX) {
	    n = KCACHE_NAME_MAX - 1;
	}

	for (size_t i = 0; i < n; i++) {
	    c->name[i] = name[i];
	}

	c->object_size = object_size;
	c->chunk_size = chunk;
	c->ctor = ctor;
	c->chunks = NULL;
	c->free_head = NULL;
	c->free_count = 0;
	c->total_objects = 0;

	cache_register(c);

	return c;
}

void kcache_destroy(kcache_t *c)
{
	if (c == NULL) {
	    return;
	}

	cache_lock();

	kcache_chunk_t *ch = c->chunks;

	while (ch != NULL) {
	    kcache_chunk_t *next = ch->next;

	    buddy_free((void *)(uintptr_t)ch->va);
	    heap_free(ch);

	    ch = next;
	}

	c->chunks = NULL;
	c->free_head = NULL;
	c->free_count = 0;
	c->total_objects = 0;

	cache_unlock();

	heap_free(c);
}

void *kcache_alloc(kcache_t *c)
{
	if (c == NULL) {
	    return NULL;
	}

	cache_lock();

	kcache_mag_t *mag = &c->mags[per_cpu_id()];

	if (mag->count > 0) {
	    void *obj = mag->objs[--mag->count];
	    cache_unlock();
	    return obj;
	}

	if (c->free_head == NULL) {
	    if (!kcache_grow(c)) {
	        cache_unlock();
	        return NULL;
	    }
	}

	void *obj = c->free_head;
	c->free_head = *(void **)obj;
	c->free_count--;

	cache_unlock();

	return obj;
}

void *kcache_zalloc(kcache_t *c)
{
	void *obj = kcache_alloc(c);

	if (obj == NULL) {
	    return NULL;
	}

	cache_memset(obj, 0, c->object_size);

	return obj;
}

void kcache_free(kcache_t *c, void *ptr)
{
	if (c == NULL || ptr == NULL) {
	    return;
	}

	cache_lock();

	kcache_mag_t *mag = &c->mags[per_cpu_id()];

	if (mag->count < KCACHE_MAG_SIZE) {
	    mag->objs[mag->count++] = ptr;
	    cache_unlock();
	    return;
	}

	*(void **)ptr = c->free_head;
	c->free_head = ptr;
	c->free_count++;

	cache_unlock();
}

size_t kcache_object_size(kcache_t *c)
{
	if (c == NULL) {
	    return 0;
	}

	return c->object_size;
}

size_t kcache_free_count(kcache_t *c)
{
	if (c == NULL) {
	    return 0;
	}

	return c->free_count;
}

size_t kcache_total_count(kcache_t *c)
{
	if (c == NULL) {
	    return 0;
	}

	return c->total_objects;
}