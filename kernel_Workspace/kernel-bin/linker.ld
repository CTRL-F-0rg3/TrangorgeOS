/* kernel_Workspace/kernel-bin/linker.ld */

ENTRY(_start)

KERNEL_VMA = 0xFFFFFFFF80000000;
KERNEL_PAGE_OFFSET = KERNEL_VMA >> 12;

SECTIONS
{
    . = 1M; /* Load address for physical mapping (bootloader maps first 2MB) */

    .text BLOCK(4K) : ALIGN(4K)
    {
        __text_start = .;
        *(.multiboot)      /* Multiboot header must be in the first 8KB */
        *(.text .text.*)
        __text_end = .;
    }

    .rodata BLOCK(4K) : ALIGN(4K)
    {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data BLOCK(4K) : ALIGN(4K)
    {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss BLOCK(4K) : ALIGN(4K)
    {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        __bss_end = .;
    }

    __kernel_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.comment)
    }
}