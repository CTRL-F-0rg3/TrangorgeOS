#ifndef MM_CACHE_CACHE_H
#define MM_CACHE_CACHE_H

#include <stdbool.h>
#include "object_cashe.h"

bool cache_init(void);
bool cache_ready(void);

void cache_register(kcache_t *c);
kcache_t *cache_find(const char *name);

void cache_dump(void);

#endif