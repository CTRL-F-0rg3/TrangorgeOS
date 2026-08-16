#include "address.h"
#include "../arch/x86_64/memory.h"

bool addr_is_canonical(uint64_t a)
{
    uint64_t top = a >> 47;

    return top == 0 || top == 0x1FFFF;
}

bool addr_is_user(uint64_t a)
{
    return addr_is_canonical(a) && a < ADDR_USER_MAX;
}

bool addr_is_kernel(uint64_t a)
{
    return addr_is_canonical(a) && a >= ADDR_KERNEL_MIN;
}

bool addr_is_direct_map(uint64_t a)
{
    return a >= ARCH_DIRECT_MAP_BASE && a < ARCH_DIRECT_MAP_BASE + (1ULL << 46);
}

uint64_t addr_align_up(uint64_t a, uint64_t align)
{
    if (align == 0) {
        return a;
    }

    uint64_t mask = align - 1;

    if (a > UINT64_MAX - mask) {
        return UINT64_MAX;
    }

    return (a + mask) & ~mask;
}

uint64_t addr_align_down(uint64_t a, uint64_t align)
{
    if (align == 0) {
        return a;
    }

    return a & ~(align - 1);
}

uint64_t addr_phys_to_direct(uint64_t phys)
{
    return ARCH_DIRECT_MAP_BASE + phys;
}

uint64_t addr_direct_to_phys(uint64_t va)
{
    return va - ARCH_DIRECT_MAP_BASE;
}

const char *addr_describe(uint64_t a)
{
    if (!addr_is_canonical(a)) {
        return "noncanonical";
    }

    if (addr_is_direct_map(a)) {
        return "direct";
    }

    if (addr_is_user(a)) {
        return "user";
    }

    if (addr_is_kernel(a)) {
        return "kernel";
    }

    return "canonical";
}