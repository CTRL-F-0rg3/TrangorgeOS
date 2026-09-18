#ifndef MM_CORE_ADDRESS_H
#define MM_CORE_ADDRESS_H

#include <stdint.h>
#include <stdbool.h>

#define ADDR_USER_MAX   0x0000800000000000ULL
#define ADDR_KERNEL_MIN 0xFFFF800000000000ULL

bool addr_is_canonical(uint64_t a);
bool addr_is_user(uint64_t a);
bool addr_is_kernel(uint64_t a);
bool addr_is_direct_map(uint64_t a);

uint64_t addr_align_up(uint64_t a, uint64_t align);
uint64_t addr_align_down(uint64_t a, uint64_t align);

uint64_t addr_phys_to_direct(uint64_t phys);
uint64_t addr_direct_to_phys(uint64_t va);

const char *addr_describe(uint64_t a);

#endif