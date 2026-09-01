.section .text

.global tlb_asm_invlpg
.type tlb_asm_invlpg, @function
tlb_asm_invlpg:
    invlpg (%rdi)
    ret

.global tlb_asm_read_cr3
.type tlb_asm_read_cr3, @function
tlb_asm_read_cr3:
    mov %cr3, %rax
    ret

.global tlb_asm_write_cr3
.type tlb_asm_write_cr3, @function
tlb_asm_write_cr3:
    mov %rdi, %cr3
    ret

.global tlb_asm_read_cr4
.type tlb_asm_read_cr4, @function
tlb_asm_read_cr4:
    mov %cr4, %rax
    ret

.global tlb_asm_write_cr4
.type tlb_asm_write_cr4, @function
tlb_asm_write_cr4:
    mov %rdi, %cr4
    ret


.global tlb_asm_invpcid
.type tlb_asm_invpcid, @function
tlb_asm_invpcid:
    .byte 0x66, 0x0F, 0x38, 0x82, 0x3E
    ret

.global tlb_asm_wbinvd
.type tlb_asm_wbinvd, @function
tlb_asm_wbinvd:
    wbinvd
    ret

.global tlb_asm_clflush
.type tlb_asm_clflush, 
tlb_asm_clflush:
    clflush (%rdi)
    ret