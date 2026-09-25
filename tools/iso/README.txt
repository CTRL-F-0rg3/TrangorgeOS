TrangorgeOS @VERSION@ — x86_64 demo image
=========================================

This is an ALPHA pre-release demo build of TrangorgeOS, a from-scratch
bare-metal operating system (Separated Tri-Partition Architecture).

HOW TO BOOT
-----------
Legacy BIOS only (no UEFI). Choose one of:

  * CD/DVD .......... boot the disc directly (ISOLINUX + MEMDISK).
  * USB stick ....... the ISO is hybrid: `dd if=...iso of=/dev/sdX bs=4M`
                       then boot the stick in legacy/CSM mode.
  * QEMU ............ tools/run-iso.sh (in the source tree)

WHAT YOU SHOULD SEE
-------------------
  1. ISOLINUX/MEMDISK banner, then the bootloader-0.9 loading screen.
  2. Serial (COM1, 115200 8N1) and VGA/framebuffer kernel log:
     [mm] allocator initialized OK, [gfx], [pci], [nic], [fs], [cpu] ...
  3. The built-in self-test suite (per-module PASS/FAIL report).
  4. The in-kernel text terminal.

The file-system self-test needs a writable ATA disk (it formats it with TFS).
Booting from a CD on a real machine without any IDE/SATA disk will therefore
show an [fs] failure - that is expected and does not stop the system.

Attach a writable disk (see tools/run-iso.sh) to get the full green report.

CONTENTS
--------
  /VERSION.TXT            build version string
  /boot/trangorge.img     the raw bootloader-0.9 kernel image (boot payload)
  /isolinux/              ISOLINUX + MEMDISK bootloader files

SOURCE / LICENCE
----------------
  Repository : https://github.com/CTRL-F-0rg3/TrangorgeOS
  Branch     : stabilizing (alpha)
  Licence    : see AGPR_V3 in the repository root

Built with the `tools/mkiso.sh` script; see `tools/iso/README.md` for the exact
build recipe and the list of known limitations.
