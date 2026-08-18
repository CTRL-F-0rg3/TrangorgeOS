ENTRY(ds_entry)

SECTIONS
{
    . = 0x40005000;

    .text : {
        KEEP(*(.text.ds_entry))
        *(.text .text.*)
    }

    .rodata : { *(.rodata .rodata.*) }
    .data   : { *(.data .data.*) }
    .bss    : { *(.bss .bss.*) }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note*)
        *(.comment)
    }
}