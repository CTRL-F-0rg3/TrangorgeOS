#ifndef MM_CACHE_OBJECT_CACHE_H
#define MM_CACHE_OBJECT_CACHE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "per_cpu.h"

#define KCACHE_NAME_MAX 32
#define KCACHE_MAG_SIZE 16

typedef void (*kcache_ctor_t)(void *obj);

typedef struct kcache_chunk {
    struct kcache_chunk *next;
    uint64_t va;
} kcache_chunk_t;

typedef struct kcache_mag {
    void *objs[KCACHE_MAG_SIZE];
    size_t count;
} kcache_mag_t;

typedef struct kcache {
    char name[KCACHE_NAME_MAX];

    size_t object_size;
    size_t chunk_size;

    kcache_ctor_t ctor;

    kcache_chunk_t *chunks;

    void *free_head;
    size_t free_count;
    size_t total_objects;

    kcache_mag_t mags[PER_CPU_MAX];

    struct kcache *next;
} kcache_t;

kcache_t *kcache_create(const char *name,
                        size_t object_size,
                        kcache_ctor_t ctor);

void kcache_destroy(kcache_t *c);

void *kcache_alloc(kcache_t *c);
void *kcache_zalloc(kcache_t *c);
void kcache_free(kcache_t *c, void *ptr);

size_t kcache_object_size(kcache_t *c);
size_t kcache_free_count(kcache_t *c);
size_t kcache_total_count(kcache_t *c);

#endif