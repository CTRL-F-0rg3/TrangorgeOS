/* comgrub/linker.ld */
ENTRY(_start)

SECTIONS {
    . = 1M;

    .text : ALIGN(4K) {
        /* Multiboot header MUSI być w pierwszych 8KB pliku */
        KEEP(*(.multiboot))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
}