.section .text

.set TRAMPOLINE_BASE, 0x8000

.global trampoline_start
.global trampoline_end
.global trampoline_cr3
.global trampoline_entry
.global trampoline_stack
.global trampoline_arg

.code16
trampoline_start:
    cli
    cld

    mov %cs, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    lgdt %cs:(trampoline_gdtr - trampoline_start)

    mov %cr0, %eax
    or $1, %eax
    mov %eax, %cr0

    ljmpl $0x08, $(TRAMPOLINE_BASE + (trampoline_32 - trampoline_start))

.code32
trampoline_32:

    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    mov %cr4, %eax
    or $((1 << 5) | (1 << 9) | (1 << 10)), %eax
    mov %eax, %cr4

    movl $(TRAMPOLINE_BASE + (trampoline_cr3 - trampoline_start)), %eax
    mov (%eax), %eax
    mov %eax, %cr3

    mov $0xC0000080, %ecx
    rdmsr
    or $(1 << 8), %eax
    or $(1 << 11), %eax
    wrmsr

    mov %cr0, %eax
    or $(1 << 31), %eax
    mov %eax, %cr0

    ljmpl $0x18, $(TRAMPOLINE_BASE + (trampoline_64 - trampoline_start))

.code64
trampoline_64:

    mov $0x20, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    xor %eax, %eax
    mov %ax, %fs
    mov %ax, %gs

    movabs $(TRAMPOLINE_BASE + (trampoline_arg - trampoline_start)), %rcx
    mov (%rcx), %rdi

    movabs $(TRAMPOLINE_BASE + (trampoline_stack - trampoline_start)), %rcx
    mov (%rcx), %rsp

    movabs $(TRAMPOLINE_BASE + (trampoline_entry - trampoline_start)), %rcx
    mov (%rcx), %rax

    jmp *%rax

.align 8
trampoline_gdt:
    .quad 0x0000000000000000
    .quad 0x00CF9A000000FFFF
    .quad 0x00CF92000000FFFF
    .quad 0x00AF9A000000FFFF
    .quad 0x00AF92000000FFFF
trampoline_gdt_end:

.align 4
trampoline_gdtr:
    .word (trampoline_gdt_end - trampoline_gdt - 1)
    .long (TRAMPOLINE_BASE + (trampoline_gdt - trampoline_start))

.align 8
trampoline_cr3:    .quad 0
trampoline_entry:  .quad 0
trampoline_stack:  .quad 0
trampoline_arg:    .quad 0

.global trampoline_end
trampoline_end:
    .quad 0
