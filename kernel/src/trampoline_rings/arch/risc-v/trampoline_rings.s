.section .data
.align 3
kstack_top:
    .dword 0

.section .text
.align 2

/*
 * CpuCtx:
 *   x0..x31 : 0..248   (x0 nieużywane)
 *   sp      : 256
 *   epc     : 264
 *   sstatus : 272
 *   pad     : 280
 *   size    : 288
 */

.global trap_entry
trap_entry:
    csrrw sp, sscratch, sp        # sp = kernel, sscratch = world sp
    addi sp, sp, -288

    sd x1,   8(sp)
    sd x3,   24(sp)
    sd x4,   32(sp)
    sd x5,   40(sp)
    sd x6,   48(sp)
    sd x7,   56(sp)
    sd x8,   64(sp)
    sd x9,   72(sp)
    sd x10,  80(sp)
    sd x11,  88(sp)
    sd x12,  96(sp)
    sd x13,  104(sp)
    sd x14,  112(sp)
    sd x15,  120(sp)
    sd x16,  128(sp)
    sd x17,  136(sp)
    sd x18,  144(sp)
    sd x19,  152(sp)
    sd x20,  160(sp)
    sd x21,  168(sp)
    sd x22,  176(sp)
    sd x23,  184(sp)
    sd x24,  192(sp)
    sd x25,  200(sp)
    sd x26,  208(sp)
    sd x27,  216(sp)
    sd x28,  224(sp)
    sd x29,  232(sp)
    sd x30,  240(sp)
    sd x31,  248(sp)

    csrr t0, sscratch
    sd t0, 256(sp)                /* world sp */
    csrr t0, sepc
    sd t0, 264(sp)
    csrr t0, sstatus
    sd t0, 272(sp)

    mv a0, sp
    call tr_hyper

    /* powrót bez przełączenia świata */
    ld t0, 264(sp)
    csrw sepc, t0
    ld t0, 272(sp)
    csrw sstatus, t0
    ld t0, 256(sp)
    csrw sscratch, t0

    ld x1,   8(sp)
    ld x3,   24(sp)
    ld x4,   32(sp)
    ld x5,   40(sp)
    ld x6,   48(sp)
    ld x7,   56(sp)
    ld x8,   64(sp)
    ld x9,   72(sp)
    ld x10,  80(sp)
    ld x11,  88(sp)
    ld x12,  96(sp)
    ld x13,  104(sp)
    ld x14,  112(sp)
    ld x15,  120(sp)
    ld x16,  128(sp)
    ld x17,  136(sp)
    ld x18,  144(sp)
    ld x19,  152(sp)
    ld x20,  160(sp)
    ld x21,  168(sp)
    ld x22,  176(sp)
    ld x23,  184(sp)
    ld x24,  192(sp)
    ld x25,  200(sp)
    ld x26,  208(sp)
    ld x27,  216(sp)
    ld x28,  224(sp)
    ld x29,  232(sp)
    ld x30,  240(sp)
    ld x31,  248(sp)

    addi sp, sp, 288
    csrrw sp, sscratch, sp        # sp = world, sscratch = kernel
    sret

/* tr_init(kernel_stack_top) */
.global tr_init
.type tr_init, @function
tr_init:
    la t0, kstack_top
    sd a0, 0(t0)

    csrw sscratch, a0

    la t0, trap_entry
    csrw stvec, t0

    /* SUM=1: kernel może czytać strony U w tr_hyper (logi itd.) */
    li t0, (1 << 18)
    csrs sstatus, t0

    ret

/* tr_restore_ctx(a0 = CpuCtx*) — wejście/wznowienie świata */
.global tr_restore_ctx
.type tr_restore_ctx, @function
tr_restore_ctx:
    ld t0, 264(a0)
    csrw sepc, t0
    ld t0, 272(a0)
    csrw sstatus, t0

    la t0, kstack_top
    ld t0, 0(t0)
    csrw sscratch, t0

    ld sp, 256(a0)

    ld x1,  8(a0)
    ld x3,  24(a0)
    ld x4,  32(a0)
    ld x5,  40(a0)
    ld x6,  48(a0)
    ld x7,  56(a0)
    ld x8,  64(a0)
    ld x9,  72(a0)
    ld x11, 88(a0)
    ld x12, 96(a0)
    ld x13, 104(a0)
    ld x14, 112(a0)
    ld x15, 120(a0)
    ld x16, 128(a0)
    ld x17, 136(a0)
    ld x18, 144(a0)
    ld x19, 152(a0)
    ld x20, 160(a0)
    ld x21, 168(a0)
    ld x22, 176(a0)
    ld x23, 184(a0)
    ld x24, 192(a0)
    ld x25, 200(a0)
    ld x26, 208(a0)
    ld x27, 216(a0)
    ld x28, 224(a0)
    ld x29, 232(a0)
    ld x30, 240(a0)
    ld x31, 248(a0)
    ld x10, 80(a0)                /* a0 na samym końcu */

    sret