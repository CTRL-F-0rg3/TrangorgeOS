OUTPUT_ARCH(riscv)
ENTRY(_start)

/* QEMU virt: kernel placed right after OpenSBI (0x8020_0000). */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;

    /* Boot and entry code, kept first so `_start` lands on load address. */
    .text : {
        KEEP(*(.text.entry))
        *(.text*)
    }

    . = ALIGN(4096);
    .rodata : { *(.rodata* .rodata.*) }

    . = ALIGN(4096);
    .data : { *(.data* .data.*) }

    . = ALIGN(4096);
    .sdata : {
        __global_pointer$ = . + 0x800;
        *(.sdata* .sdata.*)
    }

    . = ALIGN(4096);
    .bss : {
        __bss_start = .;
        *(.bss* .sbss*) *(COMMON)
        __bss_end = .;
    }

    /* Boot stack grows down from _stack_top. */
    . = ALIGN(4096);
    . = . + 0x100000;          /* 1 MiB bootstrap stack */
    _stack_top = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.note*)
    }
}