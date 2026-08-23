#include "cache.h"

extern void kprintf(const char *fmt, ...);

static kcache_t *cache_registry = NULL;
static bool cache_initialized = false;

bool cache_init(void)
{
	if (cache_initialized) {
	    return true;
	}

	per_cpu_init();

	cache_registry = NULL;
	cache_initialized = true;

	return true;
}

bool cache_ready(void)
{
	return cache_initialized;
}

void cache_register(kcache_t *c)
{
	if (c == NULL) {
	    return;
	}

	c->next = cache_registry;
	cache_registry = c;
}

kcache_t *cache_find(const char *name)
{
	kcache_t *c = cache_registry;

	while (c != NULL) {
	    const char *a = c->name;
	    const char *b = name;

	    while (*a != '\0' && *a == *b) {
	        a++;
	        b++;
	    }

	    if (*a == '\0' && *b == '\0') {
	        return c;
	    }

	    c = c->next;
	}

	return NULL;
}

void cache_dump(void)
{
	if (!cache_initialized) {
	    kprintf("CACHE: not initialized\n");
	    return;
	}

	kprintf("CACHE:\n");

	kcache_t *c = cache_registry;

	while (c != NULL) {
	    kprintf("  %-16s obj=%u free=%u total=%u\n",
	            c->name,
	            (unsigned int)c->object_size,
	            (unsigned int)c->free_count,
	            (unsigned int)c->total_objects);

	    c = c->next;
	}
}