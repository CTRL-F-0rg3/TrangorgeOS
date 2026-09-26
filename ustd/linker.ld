/* tgs-init — userspace linker script (bare ELF, no C runtime).
 *
 * Paired with `targets/x86_64-trangorge-uspace.json`. Everything is static:
 * there is no dynamic loader, no libc.so and no crt1.o — the entry point is
 * `tgs_rt::_start`, which sets up the stack and calls `main(argc, argv)`.
 *
 * Layout (x86_64, non-PIE, loaded at the identity address the kernel's ELF
 * loader maps PT_LOAD segments into):
 *
 *   0x0040_0000  .text      the program itself
 *   0x0060_0000  .rodata
 *   0x0061_0000  .tdata / .tbss   (single-threaded: plain data, see README)
 *   0x0062_0000  .data / .bss      (std's globals, the malloc arena, errno)
 *
 * Everything above 0x0040_0000 is left free for `mmap`, so the heap can grow
 * upwards without ever colliding with the image.
 */

ENTRY(_start)

SECTIONS
{
    . = 0x0040_0000;
    _image_start = .;

    .text : ALIGN(4K)
    {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
        *(.gcc_except_table .gcc_except_table.*)
    }

    /* Thread-local storage. The kernel does not program `fs_base` yet, so with
     * a single thread the TLS block is just ordinary data at a fixed address.
     * `tgs-rt` refuses to spawn threads until the kernel grows `fs_base`. */
    .tdata : ALIGN(4K)
    {
        *(.tdata .tdata.*)
    }
    .tbss : ALIGN(4K)
    {
        *(.tbss .tbss.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
        *(.data.rel.ro .data.rel.ro.*)
        *(.got .got.plt)
    }

    .bss (NOLOAD) : ALIGN(4K)
    {
        *(.bss .bss.*)
        *(COMMON)
    }

    . = ALIGN(4K);
    _image_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.note.GNU-stack)
        *(.note.gnu.property)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
