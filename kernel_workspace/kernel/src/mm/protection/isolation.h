#ifndef MM_PROTECTION_ISOLATION_H
#define MM_PROTECTION_ISOLATION_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

bool isolation_init(void);

bool isolation_has_smep(void);
bool isolation_has_smap(void);
bool isolation_enable_smep(void);
bool isolation_enable_smap(void);

size_t isolation_audit_kernel(void);

#endif