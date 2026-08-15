#ifndef MM_CACHE_PER_CPU_H
#define MM_CACHE_PER_CPU_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifndef PER_CPU_MAX
#define PER_CPU_MAX 1
#endif

bool per_cpu_init(void);
size_t per_cpu_count(void);
size_t per_cpu_id(void);

#endif