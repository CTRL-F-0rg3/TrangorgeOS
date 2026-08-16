#ifndef ARCH_X86_64_TLB_ASM_H
#define ARCH_X86_64_TLB_ASM_H

#include <stdint.h>

void tlb_asm_invlpg(uint64_t addr);

uint64_t tlb_asm_read_cr3(void);
void tlb_asm_write_cr3(uint64_t value);

uint64_t tlb_asm_read_cr4(void);
void tlb_asm_write_cr4(uint64_t value);

void tlb_asm_invpcid(uint64_t type, const void *desc);

void tlb_asm_wbinvd(void);
void tlb_asm_clflush(uint64_t addr);

#endif