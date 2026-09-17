.section .data
.align 16
.global tr_gdt
tr_gdt:
    .quad 0x0000000000000000
    .quad 0x00209A0000000000      
    .quad 0x0000920000000000      
    .quad 0x0020BA0000000000      
    .quad 0x0000B20000000000      
    .quad 0x0020FA0000000000      
    .quad 0x0000F20000000000      
.global tr_tss_desc
tr_tss_desc:
    .space 16

gdt_ptr:
    .short (gdt_ptr - tr_gdt) - 1
    .quad tr_gdt

.align 16
.global tr_tss
tr_tss:
    .space 104

.section .text

.global tr_init
.type tr_init, @function
tr_init:
    mov %rdi, %rax
    mov %eax, (tr_tss + 4)        
    shr $32, %rax
    mov %eax, (tr_tss + 8)        

    lgdt gdt_ptr

    mov $0x10, %ax
    mov %ax, %ss
    mov %ax, %ds
    mov %ax, %es

    pushq $0x08
    leaq 1f(%rip), %rax
    pushq %rax
    lretq
1:
    mov $0x38, %ax
    ltr %ax

    ret



.global isr_hypercall
.type isr_hypercall, @function
isr_hypercall:
    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi
    push %rdi
    push %rbp
    push %r8
    push %r9
    push %r10
    push %r11
    push %r12
    push %r13
    push %r14
    push %r15

    mov %rsp, %rdi
    call tr_hyper

    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rbp
    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax

    iretq

.global isr_default
.type isr_default, @function
isr_default:
    iretq

.global tr_restore_ctx
.type tr_restore_ctx, @function
tr_restore_ctx:
    mov %rdi, %rsp
    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rbp
    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    iretq