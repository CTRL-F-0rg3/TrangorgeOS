#ifndef MM_CORE_RANGE_H
#define MM_CORE_RANGE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>


bool range_from_addr_len(uint64_t addr,
                         uint64_t len,
                         uint64_t align,
                         uint64_t limit_min,
                         uint64_t limit_max,
                         bool require_canonical,
                         uint64_t *out_start,
                         uint64_t *out_end);

/* Sprawdza, czy adres jest kanoniczny wg reguł x86_64 (bity 63:47). */
bool range_is_canonical(uint64_t addr);

#endif