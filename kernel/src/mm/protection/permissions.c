#include "permissions.h"

static bool strict_wx = true;

bool perm_is_wx(uint32_t prot)
{
    return (prot & PROT_WRITE) && (prot & PROT_EXEC);
}

uint32_t perm_sanitize(uint32_t prot)
{
    if (perm_is_wx(prot)) {
        prot &= ~PROT_EXEC;
    }

    return prot;
}

bool perm_mprotect_allowed(uint32_t old_prot, uint32_t new_prot)
{
    if (perm_is_wx(new_prot)) {
        return false;
    }

    if (strict_wx &&
        (old_prot & PROT_WRITE) &&
        (new_prot & PROT_EXEC)) {
        return false;
    }

    return true;
}

void perm_set_strict_wx(bool on)
{
    strict_wx = on;
}

uint32_t perm_kernel_default(void)
{
    return PROT_READ | PROT_WRITE;
}

uint32_t perm_user_default(void)
{
    return PROT_READ | PROT_WRITE;
}