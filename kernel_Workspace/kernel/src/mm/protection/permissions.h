#ifndef MM_PROTECTION_PERMISSIONS_H
#define MM_PROTECTION_PERMISSIONS_H

#include <stdint.h>
#include <stdbool.h>
#include "../paging/paging.h"

uint32_t perm_sanitize(uint32_t prot);
bool perm_is_wx(uint32_t prot);
bool perm_mprotect_allowed(uint32_t old_prot, uint32_t new_prot);
void perm_set_strict_wx(bool on);

uint32_t perm_kernel_default(void);
uint32_t perm_user_default(void);

#endif