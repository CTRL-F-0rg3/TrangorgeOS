

MBOOT2_MAGIC    equ 0xE85250D6
MBOOT2_ARCH     equ 0
MBOOT2_LENGTH   equ (mboot_header_end - mboot_header_start)
MBOOT2_CHECKSUM equ -(MBOOT2_MAGIC + MBOOT2_ARCH + MBOOT2_LENGTH)

section .multiboot
align 8
mboot_header_start:
    dd MBOOT2_MAGIC
    dd MBOOT2_ARCH
    dd MBOOT2_LENGTH
    dd MBOOT2_CHECKSUM

    dw 0
    dw 0
    dd 8
mboot_header_end:

section .bss
align 16
stack_bottom:
    resb 65536            :

section .text
bits 32

global _start
extern kernel_main

_start:
    mov esp, stack_top

    push ebx
    push eax

    call kernel_main

.hang:
    cli
    hlt
    jmp .hang