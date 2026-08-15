#ifndef MM_PROTECTION_GUARD_H
#define MM_PROTECTION_GUARD_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "../process/address_space.h"

uint64_t guard_install(proc_aspace_t *pa, uint64_t addr, size_t len);
bool guard_user_stack(proc_aspace_t *pa);

#endif