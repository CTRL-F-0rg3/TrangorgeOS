

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

    ; Tagi — terminator
    dw 0
    dw 0
    dd 8
mboot_header_end:

section .bss
align 16
stack_bottom:
    resb 65536            ; 64 KB stosu
stack_top:

section .text
bits 32

global _start
extern kernel_main        ; wyeksportowane z kernel crate

_start:
    ; Ustaw stos
    mov esp, stack_top

    ; Przekaż magic i info do kernela
    push ebx              ; multiboot2 info pointer
    push eax              ; multiboot2 magic

    ; Wywołaj kernel
    call kernel_main

    ; Jeśli kernel wróci — halt
.hang:
    cli
    hlt
    jmp .hang