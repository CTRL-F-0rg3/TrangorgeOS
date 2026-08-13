bits 64

section .bss
buffer: resb 16

section .text
global main
main:
fn_main:
    mov rax, 16
    mov rbx, 32
    mov rcx, rax
    add rcx, rbx
    lea r14, [rel buffer]
    mov r15, 16
    mov r12, 0
fill_1:
    test r15, r15
    jz fill_end_2
    mov byte [r14], r12b
    inc r14
    dec r15
    jmp fill_1
fill_end_2:
    lea r14, [rel buffer]
    mov byte [r14 + 0], 65
    lea r14, [rel buffer]
    mov byte [r14 + 1], 66
    lea r14, [rel buffer]
    mov byte [r14 + 2], 67
    lea r14, [rel buffer]
    movzx rax, byte [r14 + 0]
    lea r14, [rel buffer]
    movzx rbx, byte [r14 + 1]
    mov rcx, rax
    add rcx, rbx
    cmp rcx, 131
    jne else_1
    mov rax, 1
    jmp end_2
else_1:
    mov rax, 0
end_2:
loop_3:
    cmp rax, 0
    je loop_end_4
    mov rbx, rax
    sub rbx, 1
    mov rax, rbx
    jmp loop_3
loop_end_4:
    ret
