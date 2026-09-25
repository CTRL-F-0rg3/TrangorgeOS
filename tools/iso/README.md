# TrangorgeOS — demo ISO images

Build one bootable, publishable `.iso` of the current `stabilizing` (alpha) tree.

```sh
just iso              # build  dist/TrangorgeOS-<version>-x86_64-demo.iso
just iso --release    # same, kernel built with opt-level 2
just iso-run          # boot the newest ISO in QEMU (window + serial on stdout)
tools/run-iso.sh --headless    # automated smoke test, exits with a status code
```

The scripts are plain bash; `just` is only a thin wrapper (it runs them through
`bash`, so the executable bit is optional - use `chmod +x tools/*.sh` if you want
to call them directly).

Headless runs wait up to 120 s for the kernel to reach the terminal (a cold boot
takes ~1-2 minutes in QEMU) and use QEMU's `-snapshot` so repeated/parallel runs
neither lock nor modify `dist/demo-root-disk.img`.

| File | Purpose |
|:---|:---|
| `../mkiso.sh` | build the kernel image, stage the ISO tree, run xorriso, write the checksum + `dist/RELEASE.txt` manifest |
| `../run-iso.sh` | boot the ISO in QEMU, optionally headless with a serial smoke test |
| `isolinux.cfg` | ISOLINUX menu (ISO `/isolinux/isolinux.cfg`) |
| `README.txt` | text file placed at the ISO root (`/README.TXT`) |
| `Dockerfile` | reproducible builder image (toolchain + xorriso + syslinux) |

## Boot path (and why it is the only one that works today)

The kernel's single working boot path is the **bootloader 0.9 BIOS image** built
by `cargo bootimage`:

```
kernel_Workspace/kernel-bin/src/main.rs  -> entry_point!(kernel_main)   (the only _start)
kernel_Workspace/kernel-bin/build.sh     -> cargo bootimage -Zbuild-std=...
kernel_Workspace/target/x86_64-kernel/<profile>/bootimage-kernel-bin.bin
```

That artefact is a *raw disk image*: a 512-byte BIOS boot sector at LBA 0 followed
by the bootloader stages and the kernel payload, all addressed through the BIOS
INT 13h interface with 512-byte sectors.

An El Torito CD is normally booted in **no-emulation** mode, where the BIOS reads
and hands over the drive in 2048-byte sectors and does not provide a disk-style
INT 13h interface. A raw 512-byte-sector boot image therefore cannot be started
that way directly. The ISO is built like this instead:

```
El Torito (no emulation)  ->  ISOLINUX  ->  MEMDISK  ->  /boot/trangorge.img
                                                                 |
                                    the unmodified bootimage, loaded to RAM
```

* **ISOLINUX** is a no-emulation El Torito bootloader that installs its own
  INT 13h handlers, which is exactly why the standard syslinux recipe works.
* **MEMDISK** claims a chunk of high memory, hooks INT 13h/INT 15h and presents
  the image as a BIOS hard disk. The raw image is ~3.9 MiB - just under MEMDISK's
  4 MiB floppy/hard-disk threshold - so the config passes `harddisk` explicitly
  instead of relying on size/geometry guessing.

The payload is byte-for-byte the same file `kernel-bin/run.sh` boots, so if the
ISO works in QEMU it proves the QEMU boot path (and vice versa).

`comgrub/` (multiboot2) and `comlimine/` (Limine) are **not** usable yet: neither
provides the `kernel_main(magic, info)` symbol the kernel actually exports
(`kernel::kernel_main(&BootInfo)` from bootloader 0.9), and neither hand-off
produces a `bootloader::BootInfo`. See `TrangorgeOS — TODO.md`, section 2.6.

## Requirements

```sh
sudo apt-get install -y xorriso syslinux-common   # Debian/Ubuntu (Ubuntu 24.04 used here)
sudo dnf install -y xorriso syslinux              # Fedora/RHEL
sudo pacman -S --needed libisoburn syslinux       # Arch
```

Plus the Rust toolchain from `rust-toolchain.toml` (`nightly-2026-08-01`,
`rust-src`, `llvm-tools-preview`) and `cargo-bootimage ^0.10`. `mkiso.sh` installs
`cargo-bootimage` on demand and can be checked with `tools/mkiso.sh --check-deps`.

`mkiso.sh` probes the usual syslinux locations on its own:
`/usr/lib/ISOLINUX` and `/usr/lib/syslinux/modules/bios` (Debian/Ubuntu),
`/usr/lib/syslinux/bios` (Arch/Manjaro), `/usr/share/syslinux`. Set
`SYSLINUX_DIR=/path/to/flat/dir` to point it at any directory containing
`isolinux.bin`, `ldlinux.c32`, `memdisk` and (optionally) `isohdpfx.bin` - that is
the escape hatch when the distro does not package them or when root is not
available:

```sh
curl -LO https://www.kernel.org/pub/linux/utils/boot/syslinux/syslinux-6.03.tar.xz
tar -xJf syslinux-6.03.tar.xz
mkdir -p /tmp/syslinux-flat && cp \
  syslinux-6.03/bios/core/isolinux.bin \
  syslinux-6.03/bios/com32/elflink/ldlinux/ldlinux.c32 \
  syslinux-6.03/bios/memdisk/memdisk \
  syslinux-6.03/bios/mbr/isohdpfx.bin /tmp/syslinux-flat/
SYSLINUX_DIR=/tmp/syslinux-flat tools/mkiso.sh
```

## What the build produces

```
dist/
  TrangorgeOS-<version>-x86_64-demo.iso          the image (hybrid MBR: CD or dd to USB)
  TrangorgeOS-<version>-x86_64-demo.iso.sha256   checksum
  RELEASE.txt                                    release manifest (version, git commit, hashes)
  iso-root/                                      staged ISO9660 tree (for inspection)
  demo-root-disk.img                             writable ATA disk for local QEMU runs
  serial-*.log                                   serial capture of the last QEMU run
```

Version resolution order: `--version=` → `$TRANGORGE_VERSION` → `./VERSION` file →
`git describe --tags` → `dev-<short-sha>`.

## Publishing checklist

1. `just iso --release` on a clean checkout.
2. `tools/run-iso.sh --headless` — the smoke test must print `smoke test PASSED`
   (it greps the serial log for the kernel banner, the allocator, the self-test
   suite and the `SYSTEM STATUS:` line).
3. Verify the manifest: `cat dist/RELEASE.txt`.
4. Attach to the GitHub release: the `.iso`, its `.sha256` and `RELEASE.txt`.
5. Paste the checksum and the `Git commit` line into the release notes so the
   artefact can be tied to a revision.

```sh
(cd dist && sha256sum -c TrangorgeOS-*-demo.iso.sha256)
```

## Verified status

Build and boot verified on 2026-09-25 (`stabilizing` @ `8393847`, Manjaro/Arch host,
xorriso 1.5.8, syslinux 6.03 assets, cargo bootimage 0.10.5):

```
$ tools/mkiso.sh --no-build
[mkiso] ISO size: 5 MiB
dist/TrangorgeOS-dev-8393847-x86_64-demo.iso
sha256 422d2d99965f481afdd837886fc85b6ae183b151cd391c0394f730294665326a

$ tools/run-iso.sh --headless
  [ OK ] kernel banner reached
  [ OK ] memory allocator online
  [ OK ] self-test suite ran
  -> SYSTEM STATUS: 19/19 OK
```

* The **memdisk** ISO boots in QEMU: the kernel log on COM1 ends with
  `Welcome in my Galaxy!`, `SYSTEM STATUS: 19/19 OK` and the in-kernel terminal
  prompt (including the TFS format + write/read roundtrip on the attached ATA disk).
* A cold boot takes roughly **1-2 minutes** under QEMU (SMP bring-up, PCI/USB/NIC
  probing and the 19 module self-tests), hence the 120 s headless timeout.
* `--boot=hdemul` produced **no** boot under SeaBIOS - El Torito hard-disk
  emulation never reached the bootloader's boot sector (empty serial log, QEMU
  exits after the timeout). It is kept only as a syslinux-free fallback; the
  MEMDISK path is the supported one.

## Known limitations

* **Legacy BIOS only.** No UEFI/CSM-free boot: bootloader 0.9 is BIOS-only.
  A UEFI image needs a real Limine/GRUB hand-off (see TODO 2.6).
* **No persistence from the CD.** TFS is read/write only on ATA disks; a CD is
  read-only and ATAPI is not driven by the kernel, so the fs self-test reports
  `FAILED` when no disk is attached (the system keeps running).
  `run-iso.sh` always attaches a writable disk to keep the report green.
* **MEMDISK loads the image into RAM** (~8 MiB of high memory for the current
  image) — irrelevant for QEMU, but it means very large kernels would need a
  different strategy.
* **`--boot=hdemul` fallback**: El Torito hard-disk emulation, no syslinux
  needed. It depends on the BIOS deriving a usable geometry from the image; the
  bootimage's MBR carries no partition table, so `--synthesize-mbr` can write an
  experimental entry at offset 446. Treat this mode as a debugging tool.
* The image is a **pre-release alpha**; `just iso` fetches nothing, so the
  resulting ISO is reproducible from the recorded git commit.
