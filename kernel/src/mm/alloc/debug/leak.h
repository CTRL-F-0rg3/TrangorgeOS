#ifndef MM_ALLOC_DEBUG_LEAK_H
#define MM_ALLOC_DEBUG_LEAK_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool leak_track(void *ptr, size_t size, uint64_t caller);

/*
 * Jak leak_track(), ale dodatkowo zwraca unikalny, monotonicznie rosnący
 * `alloc_id` przypisany tej alokacji (sekcja 5 planu: "jednoznaczne
 * śledzenie życia obiektu"). Zwraca false (bez ustawiania *out_id), gdy
 * tabela jest pełna — DOKŁADNIE ten przypadek MUSI być sprawdzony przez
 * wywołującego (patrz alloc_debug.c/dbg_alloc): zignorowanie zwracanej
 * wartości prowadzi do cichego nieśledzenia alokacji, a w konsekwencji
 * do fałszywego alarmu "double free" i trwałego wycieku pamięci przy
 * odpowiadającym `dbg_free()` (bo dawny kod przerywał się przed
 * wywołaniem heap_free() po nieudanym leak_untrack()).
 */
bool leak_track_id(void *ptr, size_t size, uint64_t caller, uint64_t *out_id);

bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller);
bool leak_contains(void *ptr);
size_t leak_count(void);

void leak_dump(void);

#endif