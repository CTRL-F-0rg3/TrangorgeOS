; kernel_Workspace/kernel-bin/src/boot.asm

global _start
extern kernel_main
extern __bss_start
extern __bss_end

section .text
bits 64

_start:
    ; 1. Clear BSS section
    mov rdi, __bss_start
    mov rcx, __bss_end
    sub rcx, rdi
    shr rcx, 3           ; Divide by 8 (clearing 64-bit words)
    xor rax, rax
    rep stosq

    ; 2. Set up a temporary boot stack (if not provided by bootloader)
    ; We allocate 64KB for the initial boot stack
    section .bss
    align 16
boot_stack_bottom:
    resb 65536
boot_stack_top:

    section .text
    lea rsp, [boot_stack_top]

    ; 3. Align stack to 16 bytes (System V AMD64 ABI requirement)
    and rsp, -16

    ; 4. Call Rust kernel_main
    ; rdi already contains the boot_info_ptr passed by the bootloader
    call kernel_main

    ; 5. Halt if kernel_main returns
.halt:
    cli
    hlt
    jmp .halt