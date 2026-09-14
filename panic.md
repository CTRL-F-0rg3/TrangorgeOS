# Kernel Hang / Panic Report — 2026-10-09
# Kernel Hang / Panic Report — LAPIC base sanity check

Key findings:
- `APIC_BASE_MSR` read reports x2APIC when bit 10 set; else LAPIC assumed at `base_phys` passed in.
- `lapic::init(base_phys)` uses `base_phys` straight as virtual if it's `>= 0xFFFF800000000000`.
- In SMP init, APs are booted with `send_startup_ipi` using `trampoline::TRAMPOLINE_BASE >> 12` as vector.

Open question:
- Where is actual LAPIC physical base loaded from? Current code does not show base-specific usage beyond `send_startup_ipi` vector.

Next active items:
1. Confirm TRAMPOLINE_BASE value and that identity mapping covers it.
2. Confirm `build.rs` linker path behavior for x86_64.
3. Add serial console capture in QEMU and reproduce hang location.


## Status
Kernel **builds and boots** in qemu (GTK display, 4 vCPUs) but **freezes during
runtime**. No console / serial output was captured from the GTK session — the VM
stays quiet and appears hung.

Attempted serial capture with qemu (`-serial file:/tmp/qemu_serial_out.txt`)
did not produce a readable guest log at the time of this report, so the exact
stop point is inferred from the boot path rather than from live kernel prints.

## Boot entry
`kernel_main(boot_info: &'static bootloader::BootInfo) -> !` (`src/main.rs`)

Sequence:
1. `init()`
2. `mm::init_from_boot_info(boot_info)`  → “[mm] allocator initialized OK” expected
3. `init_permissions()`
4. `gfx::init()`  → “[gfx] framebuffer initialized OK” expected
5. (optional) `hdmi::init::init()`
6. `pci::init()`
7. `nic::runtime::init()`
8. `bluetooth::init::init()`
9. `fs::init()`
10. **`cpu::init(boot_info)`**  ← SMP bring-up, tight waits per AP
11. `testing::run_all(TESTS)`
12. `println!("Welcome in my Galaxy!")`
13. `gfx::refresh()`, `terminal::init()`, `terminal::run()`

## Where the hang manifests
In the GTK qemu session the kernel reaches `cpu::init(boot_info)` (step 10) and
then stops producing visible progress. The SMP initialization polls for each AP
to reach its trampoline entry within a tight per-AP timeout (about 100 ms per
AP, 3 APs with 4 vCPUs). If an AP does not become ready in time, the boot path
blocks waiting for it without emitting a visible panic in the GTK display.

## Observations from the code
- SMP init lives in `src/cpu/mod.rs` / `src/cpu/smp.rs` and reaches into
  `src/cpu/scheduler/smp/mod.rs` plus the migration / stopper / balancing
  subtrees. AP boot uses a trampoline at `TRAMPOLINE_BASE`.
- The bootloader crate (`bootloader = 0.9` with `map_physical_memory` +
### Changed technical context

- Trampoline base is `TRAMPOLINE_BASE = 0x8000` in `kernel/src/cpu/trampoline.rs` and `.set TRAMPOLINE_BASE, 0x8000` in `kernel/src/cpu/trampoline.s`.
- SMP bringup uses `send_startup_ipi(apic_id, (trampoline::TRAMPOLINE_BASE >> 12) as u8)`.
- Paging init in `kernel/src/mm/arch/x86_64/paging.c` skips identity work when `boot_phys_offset == ARCH_DIRECT_MAP_BASE`, else it maps `boot_phys_offset` → `ARCH_DIRECT_MAP_BASE`.
- Kernel MM bridge is compiled by `kernel/build.rs`, but on x86_64 it does **not** inject an explicit `-T <linker.ld>`; on riscv64 it does (`riscv64-link.ld`).
- Trampoline identity map is done relative to `TRAMPOLINE_BASE` before paging is enabled.
- ACPI parsing path uses `phys_offset` for RSDP/MADT/FADT reads; MADT can carry `lapic_base_override` (type 5), current code does not show LAPIC base being initialized from MADT, only from `base_phys` passed to `lapic::init`.

### Still blocked

- Cannot reproduce and capture exact panic address or serial log yet in this environment; recommended next step is QEMU `-serial stdio` or `-serial file:` boot and bisection with early prints.

  `vga_320x200`) maps physical memory and provides `BootInfo`. The kernel
  C bridge (`libmm.a`) is compiled by `build.rs` for x86_64.
- There is **no explicit x86_64 kernel linker script** in the repo at the
- MADT is already used to read `lapic_base` in `kernel/src/cpu/acpi.rs`; `lapic_base_override` is also parsed but the AP boot path currently passes `trampoline::TRAMPOLINE_BASE >> 12` as the startup vector and does not override LAPIC base elsewhere.

  moment — `build.rs` adds `-T .../riscv64-link.ld` only for riscv64. On x86_64
  the bootloader’s default layout is used. If that layout does not match kernel
  assumptions about load address / trampoline placement, early boot can fail
  silently.
- **566 Rust warnings** remain (Rust 2024 strictness around `&mut static` /
  shared references to mutable statics in `gfx/console.rs`, `vga_buffer.rs`,
  `fs/mbr.rs`, and several scheduler files). These are not compile errors yet,
  but some are UB-adjacent and can corrupt early-boot state.
- Current `cargo run` uses the GTK display variant, so kernel `println!` /
  panic output is not visible from the host side.

## Likely root causes (ranked)
1. **SMP AP bring-up timeout / deadlock** — an AP never reaches the trampoline
   or does not complete early init within the per-AP timeout, and the BSP waits
   indefinitely (or until a timeout path that may not panic). This is the most
   probable hang location with 4 vCPUs / 3 APs.
2. **Incorrect / incomplete physical memory mapping for the kernel image or
   trampoline region** — if `BootInfo` mapping is wrong, AP bootstrap code can
   fault silently without a visible triple fault in GTK mode.
3. **Undefined behavior from `&mut static` / shared-ref patterns** in early boot
   (GFX, scheduler, FS) corrupting state that SMP init / tests depend on.
4. **Missing explicit kernel linker script for x86_64** — the bootloader
   default layout may not match kernel expectations for section / trampoline
   placement, causing early code to run at wrong addresses.

## Częste miejsce (hang point)
- `cpu::init(boot_info)` in `src/cpu/mod.rs` / `src/cpu/smp.rs` — specifically
  the per-AP wait loop after the SMP startup IPI(s). If the AP never signals
  readiness, the BSP waits here and the system hangs.

## Verification needed
- Run qemu with a **serial console** (`-serial stdio` or `-serial
  file:/tmp/qemu_serial_out.txt`) and `< /dev/null` so host can read kernel
  prints and any panic message.
- Determine whether the log stops at `[mm]`, `[gfx]`, `pci`, `nic`, `bluetooth`,
  `fs`, or at `cpu::init` / AP bring-up.
- Inspect the SMP trampoline path and per-AP early stack / GDT / IDT setup.
- Validate the bootloader-provided memory map and kernel load / trampoline
  addresses against kernel expectations.
- Consider adding an explicit x86_64 linker script if the bootloader layout is
  not what the kernel assumes.

## Verification status
- Build: clean (exit 0) for `driverspacelib`, `driverspace`, `kernel`.
- Boot: `cargo run` builds the bootimage and launches qemu.
- Runtime: hung; no serial console output captured yet in this session.

## Notes
- The GTK display session shows only qemu theme warnings; no kernel console
### Architecture-specific physical-memory offset notes

- On x86_64, `mm::init` receives `boot_phys_offset` from `boot_info.physical_memory_offset` and passes it to paging init.
- Paging code in `kernel/src/mm/arch/x86_64/paging.c` has an early exit when `boot_phys_offset == ARCH_DIRECT_MAP_BASE`; otherwise it treats the offset differently.
- If the bootloader does not map physical memory at the expected offset, the direct map and/or trampoline identity map can be wrong, which can show up as AP startup failures or early triple faults.
- The LAPIC base can come from ACPI MADT; if that value is used directly as a virtual address somewhere without the same offset assumptions as paging init, it can also cause faults.

### Actionable next steps

1. Boot the kernel with `-serial stdio` and capture the exact panic / hang point, not just the GTK display.
2. If there is no serial access yet, add an early, unconditional serial write right after paging enable and before MM-heavy init, to check whether paging itself is the fault point.
3. Compare the bootloader-provided `physical_memory_offset` with the value the kernel expects before calling `mm::init` and after paging is on.
4. If APs are not starting, focus on the trampoline region identity mapping and the startup-IPI path, and also confirm that the MADT LAPIC base is being used consistently across init.

## 2026-09-14 — objdump evidence from debug ELF (`kernel`)

- Static data (confirmed in objdump relocation output):
  - `boot_phys_offset`: `0x5c6d88`
  - `boot_phys_offset_valid`: `0x5c6d80`
- Disassembly shows explicit checks against `<boot_phys_offset_valid>` and
  `<boot_phys_offset>` in main, paging, serial, and CPU bringup paths.
- Paging init path includes early-exit / map logic matching the C/Rust preamble
  already discussed.

### Observability findings
- The debug kernel ELF (`target/x86_64-kernel/debug/kernel`) exists.
- Serial console capture remains the biggest gap:
  - `/tmp/kernel_serial_out.txt` is empty.
  - `/tmp/kernel_serial_err.txt` was too short to contain a clear panic line.

### Interpretation
- Because the binary clearly looks at `boot_phys_offset` / `boot_phys_offset_valid`
  around paging init `77b2…a5`, the most likely first failures are:
  1. Paging init fault when bootloader-provided offset is incompatible with
     `ARCH_DIRECT_MAP_BASE`, OR
  2. SMP bringup fault when AP trampoline/stack/identity mapping is inconsistent
     with LAPIC/ACPI MADT info.
- Correlation is now feasible if serial output is captured and mapped back with
  the debug ELF.

## Branch state
- `git status`: modified: `panic.md`; otherwise on `stabilizing`.
- HEAD: `183557c` (`stabilizing`).

  output is visible there.


### Current working branch and state

- Branch: `stabilizing` (tracking remote `origin/stabilizing`).
- Working tree has unstaged changes mainly in scheduler / SMP and some target/flycheck artifacts; kernel source has been modified recently in:
  - `kernel/src/cpu/scheduler/collections/mod.rs`
  - `kernel/src/cpu/scheduler/smp/migration/mod.rs`
- The most relevant panic context is in `panic.md`; the key code references are `kernel/src/main.rs`, `kernel/src/cpu/smp.rs`, `kernel/src/cpu/lapic.rs`, `kernel/src/cpu/acpi.rs`, `kernel/src/mm/mod.rs`, and the x86_64 paging/memory files.

### Current working branch and state

- Branch: `stabilizing` (tracking remote `origin/stabilizing`).
- Working tree has unstaged changes mainly in scheduler / SMP and some target/flycheck artifacts; kernel source has been modified recently in:
  - `kernel/src/cpu/scheduler/collections/mod.rs`
  - `kernel/src/cpu/scheduler/smp/migration/mod.rs`
- The most relevant panic context is in `panic.md`; the key code references are `kernel/src/main.rs`, `kernel/src/cpu/smp.rs`, `kernel/src/cpu/lapic.rs`, `kernel/src/cpu/acpi.rs`, `kernel/src/mm/mod.rs`, and the x86_64 paging/memory files.

## TODO

## References
## References

- [ ] Capture reliable serial console output with `-serial stdio`/`-serial file:`.
- [ ] Add an early unconditional serial write after paging enable, before MM/SMP init.
- [ ] Compare bootloader `physical_memory_offset` vs `ARCH_DIRECT_MAP_BASE` and
      LAPIC direct-map expectations.
- [ ] Confirm trampoline identity mapping covers AP trampoline code/stack.
- [ ] If serial log appears, correlate RIP with `kernel` ELF using addr2line/objdump.
- [ ] If offset mismatch is confirmed, decide whether to support dynamic offset or
      force early remap before paging enable.
- [ ] Fix `kernel/src/cpu/scheduler/collections/mod.rs` and
      `kernel/src/cpu/scheduler/smp/migration/mod.rs` (`pub pub mod ...`) so
      `cargo build --release` can produce the release bootimage again.



- `kernel/src/main.rs`
- `kernel/src/cpu/smp.rs`
- `kernel/src/cpu/lapic.rs`
- `kernel/src/cpu/acpi.rs`
- `kernel/src/mm/mod.rs`
- `kernel/src/mm/arch/x86_64/paging.c`
- `kernel/src/mm/arch/x86_64/memory.c`
- `kernel/src/cpu/trampoline.s`
- `kernel/src/cpu/trampoline.rs`
- `panic.md`

This report is intended as a continuation note for kernel hang debugging on x86_64 and is not yet a root-cause analysis.


- `kernel/src/main.rs`
- `kernel/src/cpu/smp.rs`
- `kernel/src/cpu/lapic.rs`
- `kernel/src/cpu/acpi.rs`
- `kernel/src/mm/mod.rs`
- `kernel/src/mm/arch/x86_64/paging.c`
- `kernel/src/mm/arch/x86_64/memory.c`
- `kernel/src/cpu/trampoline.s`
- `kernel/src/cpu/trampoline.rs`
- `panic.md`

This report is intended as a continuation note for kernel hang debugging on x86_64 and is not yet a root-cause analysis.


- [ ] Add a serial-handshake early print right after paging enable, before SMP bringup.
- [ ] Add a small test mode to boot with QEMU and a serial file/stdio in CI or locally, so the hang can be correlated to a log line.
- [ ] Compare actual bootloader-provided `physical_memory_offset` with the offset used by paging init and LAPIC direct reads.
- [ ] Confirm trampoline identity map covers at least the trampoline code and stack range used by AP bringup.

