; kernel_Workspace/kernel-bin/src/boot.asm
global kernel_bin_entry
extern kernel_bin_main

section .text
bits 64

kernel_bin_entry:
    ; Wywołaj główną funkcję Rust w kernel-bin
    call kernel_bin_main

    ; Jeśli kernel_bin_main somehow wróci
.hang:
    cli
    hlt
    jmp .hang