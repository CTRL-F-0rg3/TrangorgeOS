/* Startup trampoline for an AP (Application Processor).
 *
 * It is copied in its entirety to physical address 0x8000 (TRAMPOLINE_BASE)
 * and executed there. The code is position-independent: all data references
 * use FIXED physical addresses of the form:
 *
 *      TRAMPOLINE_BASE + (symbol - trampoline_start)
 *
 * The runtime values (cr3, entry, stack, arg) are written into the data
 * fields by the Rust code in trampoline.rs (before the IPI is sent).
 *
 * Flow: 16-bit real mode -> 32-bit protected mode -> 64-bit long mode.
 */

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

    /* ds/es/ss = cs  =>  baza segmentu = 0x8000 (fizyczny początek trampoliny) */
    mov %cs, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    /* załaduj GDT (deskryptor w segmencie cs) */
    lgdt %cs:(trampoline_gdtr - trampoline_start)

    /* włącz protected mode */
    mov %cr0, %eax
    or $1, %eax
    mov %eax, %cr0

    /* daleki skok do 32-bitowego kodu (selektor 0x08, flat, absolutny adres fizyczny) */
    ljmpl $0x08, $(TRAMPOLINE_BASE + (trampoline_32 - trampoline_start))

.code32
trampoline_32:
    /* segmenty danych 32-bit: selektor 0x10 (flat, baza 0) */
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    /* CR4: PAE + OSFXSR + OSXMMEXCPT (SSE) */
    mov %cr4, %eax
    or $((1 << 5) | (1 << 9) | (1 << 10)), %eax
    mov %eax, %cr4

    /* załaduj CR3 (fizyczny PML4 jądra) z pola danych */
    movl $(TRAMPOLINE_BASE + (trampoline_cr3 - trampoline_start)), %eax
    mov (%eax), %eax
    mov %eax, %cr3

    /* EFER: LME + NXE */
    mov $0xC0000080, %ecx
    rdmsr
    or $(1 << 8), %eax     /* LME — long mode enable */
    or $(1 << 11), %eax    /* NXE — execute-disable (bez niego PTE_NX -> #PF) */
    wrmsr

    /* włącz paging */
    mov %cr0, %eax
    or $(1 << 31), %eax
    mov %eax, %cr0

    /* daleki skok do 64-bitowego kodu (selektor 0x18, flat) */
    ljmpl $0x18, $(TRAMPOLINE_BASE + (trampoline_64 - trampoline_start))

.code64
trampoline_64:
    /* 64-bitowe segmenty danych (selektor 0x20, flat) */
    mov $0x20, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    xor %eax, %eax
    mov %ax, %fs
    mov %ax, %gs

    /* %rdi = argument (cpu_index) */
    movabs $(TRAMPOLINE_BASE + (trampoline_arg - trampoline_start)), %rcx
    mov (%rcx), %rdi

    /* %rsp = wierzchołek stosu AP */
    movabs $(TRAMPOLINE_BASE + (trampoline_stack - trampoline_start)), %rcx
    mov (%rcx), %rsp

    /* %rax = adres wejścia AP (wirtualny, górna połowa) */
    movabs $(TRAMPOLINE_BASE + (trampoline_entry - trampoline_start)), %rcx
    mov (%rcx), %rax

    jmp *%rax

/* ------------------------------------------------------------------ */
/* GDT: płaskie segmenty (limit 4 GiB, baza 0), ring 0                 */
/* ------------------------------------------------------------------ */
.align 8
trampoline_gdt:
    .quad 0x0000000000000000      /* 0x00: null */
    .quad 0x00CF9A000000FFFF      /* 0x08: 32-bit code */
    .quad 0x00CF92000000FFFF      /* 0x10: 32-bit data */
    .quad 0x00AF9A000000FFFF      /* 0x18: 64-bit code */
    .quad 0x00AF92000000FFFF      /* 0x20: 64-bit data */
trampoline_gdt_end:

.align 4
trampoline_gdtr:
    .word (trampoline_gdt_end - trampoline_gdt - 1)
    .long (TRAMPOLINE_BASE + (trampoline_gdt - trampoline_start))

/* ------------------------------------------------------------------ */
/* Pola runtime'owe (wypełniane przez trampoline.rs)                   */
/* ------------------------------------------------------------------ */
.align 8
trampoline_cr3:    .quad 0
trampoline_entry:  .quad 0
trampoline_stack:  .quad 0
trampoline_arg:    .quad 0

.global trampoline_end
trampoline_end:
    .quad 0
