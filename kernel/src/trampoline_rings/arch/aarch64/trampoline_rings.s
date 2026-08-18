.section .text

/*
 * CpuCtx:
 *   x0..x30  : 0..240
 *   sp       : 248
 *   elr      : 256
 *   spsr     : 264
 *   size     : 272
 */

.align 11
vector_table:
    /* Current EL with SP_EL0 */
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default

    /* Current EL with SP_ELx (kernel fault — na później) */
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default

    /* Lower EL AArch64 — tu ląduje svc ze światów */
    .align 7
    b vector_hyper
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default

    /* Lower EL AArch32 */
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default
    .align 7
    b vector_default

vector_default:
    eret

vector_hyper:
    sub sp, sp, #272

    stp x0, x1,   [sp, #0]
    stp x2, x3,   [sp, #16]
    stp x4, x5,   [sp, #32]
    stp x6, x7,   [sp, #48]
    stp x8, x9,   [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30,      [sp, #240]

    mrs x9, sp_el0
    str x9, [sp, #248]
    mrs x9, elr_el1
    str x9, [sp, #256]
    mrs x9, spsr_el1
    str x9, [sp, #264]

    mov x0, sp
    bl tr_hyper

    /* powrót (jeśli tr_hyper nie przełączył świata) */
    ldr x9, [sp, #248]
    msr sp_el0, x9
    ldr x9, [sp, #256]
    msr elr_el1, x9
    ldr x9, [sp, #264]
    msr spsr_el1, x9

    ldp x0, x1,   [sp, #0]
    ldp x2, x3,   [sp, #16]
    ldp x4, x5,   [sp, #32]
    ldp x6, x7,   [sp, #48]
    ldp x8, x9,   [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30,      [sp, #240]

    add sp, sp, #272
    eret

/* tr_init() — ustaw VBAR */
.global tr_init
.type tr_init, @function
tr_init:
    adr x1, vector_table
    msr vbar_el1, x1
    isb
    ret

/* tr_restore_ctx(x0 = CpuCtx*) — wejście/wznowienie świata */
.global tr_restore_ctx
.type tr_restore_ctx, @function
tr_restore_ctx:
    ldr x9, [x0, #248]
    msr sp_el0, x9
    ldr x9, [x0, #256]
    msr elr_el1, x9
    ldr x9, [x0, #264]
    msr spsr_el1, x9

    ldr x30, [x0, #240]
    ldp x28, x29, [x0, #224]
    ldp x26, x27, [x0, #208]
    ldp x24, x25, [x0, #192]
    ldp x22, x23, [x0, #176]
    ldp x20, x21, [x0, #160]
    ldp x18, x19, [x0, #144]
    ldp x16, x17, [x0, #128]
    ldp x14, x15, [x0, #112]
    ldp x12, x13, [x0, #96]
    ldp x10, x11, [x0, #80]
    ldp x8, x9,   [x0, #64]
    ldp x6, x7,   [x0, #48]
    ldp x4, x5,   [x0, #32]
    ldp x2, x3,   [x0, #16]
    ldp x0, x1,   [x0, #0]

    eret