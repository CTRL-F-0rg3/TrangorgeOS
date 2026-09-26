//! The AC97 register map, ported from the existing Odin driver.
//!
//! `drivers/audiodriver/src/odin/driver.odin` in the legacy tree already
//! described this hardware: `nam_base` (Name Base, 16-bit registers) and
//! `bm_base` (Bus Master, 8/16/32-bit registers), plus a buffer descriptor
//! list. This module keeps the same offsets, so the two can be compared
//! directly and a register change has to be made in both.
//!
//! ## Register blocks
//!
//! The Bus Master window is read and written with three accessors because
//! AC97 mixes widths in one page: `bm8`, `bm16` and `bm32`. Using the wrong one
//! silently reads the neighbouring register, which is why they are separate
//! functions rather than one generic accessor.

use core::ptr::{read_volatile, write_volatile};

/// Bus-master global control register.
pub const GLOB_CNT: u32 = 0x2C;
/// Bus-master global status register.
pub const GLOB_STA: u32 = 0x30;

/// Playback buffer descriptor list base address.
pub const PI_BDBAR: u32 = 0x00;
/// Playback last valid index.
pub const PI_LVI: u32 = 0x05;
/// Playback status.
pub const PI_SR: u32 = 0x06;
/// Playback control.
pub const PO_CR: u32 = 0x1B;

/// Capture buffer descriptor list base address.
pub const CI_BDBAR: u32 = 0x10;
/// Capture last valid index.
pub const CI_LVI: u32 = 0x15;
/// Capture status.
pub const CI_SR: u32 = 0x16;
/// Capture control.
pub const CO_CR: u32 = 0x1B;

/// Interrupt-on-completion flag in a BDL entry.
pub const IOC: u32 = 0x8000_0000;

/// Global status: buffer completion.
pub const ST_BUFSYS: u32 = 0x08;
/// Global status: end-of-buffer.
pub const ST_EOBI: u32 = 0x04;

/// One buffer descriptor list entry.
///
/// The port of `Bdl_Entry` from the Odin driver: a physical address and a
/// control word whose low bits hold the length in frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct BdlEntry {
    /// Physical address of the buffer.
    pub addr: u32,
    /// Length in frames, plus [`IOC`] when the entry ends the buffer.
    pub ctrl: u32,
}

impl BdlEntry {
    /// A non-terminal entry of `frames` frames at `addr`.
    pub const fn new(addr: u32, frames: u32) -> Self {
        Self { addr, ctrl: frames }
    }

    /// The entry that terminates a buffer descriptor list.
    pub const fn last(addr: u32, frames: u32) -> Self {
        Self { addr, ctrl: frames | IOC }
    }

    /// Length in frames, with the flag bits masked off.
    pub const fn frames(&self) -> u32 {
        self.ctrl & !IOC
    }

    /// Whether this is the terminating entry.
    pub const fn is_last(&self) -> bool {
        self.ctrl & IOC != 0
    }
}

/// A mapped AC97 register window.
///
/// The driver never holds a raw pointer to hardware: it holds the virtual base
/// that `ds-manager` granted, and every access goes through one of the
/// accessors below.
pub struct Ac97 {
    nam_base: u64,
    bm_base: u64,
}

impl Ac97 {
    /// Wrap an already-granted register window.
    ///
    /// # Safety
    ///
    /// The caller must have obtained `nam_base` and `bm_base` from
    /// `ds-manager` with `SysMapMmio` and must keep them mapped for the whole
    /// lifetime of this value.
    pub const unsafe fn new(nam_base: u64, bm_base: u64) -> Self {
        Self { nam_base, bm_base }
    }

    /// Read an 8-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page.
    pub unsafe fn bm8(&self, offset: u32) -> u8 {
        unsafe { read_volatile((self.bm_base + offset as u64) as *const u8) }
    }

    /// Write an 8-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page.
    pub unsafe fn bm8_write(&self, offset: u32, value: u8) {
        unsafe { write_volatile((self.bm_base + offset as u64) as *mut u8, value) }
    }

    /// Read a 16-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page and even.
    pub unsafe fn bm16(&self, offset: u32) -> u16 {
        unsafe { read_volatile((self.bm_base + offset as u64) as *const u16) }
    }

    /// Write a 16-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page and even.
    pub unsafe fn bm16_write(&self, offset: u32, value: u16) {
        unsafe { write_volatile((self.bm_base + offset as u64) as *mut u16, value) }
    }

    /// Read a 32-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page and 32-bit aligned.
    pub unsafe fn bm32(&self, offset: u32) -> u32 {
        unsafe { read_volatile((self.bm_base + offset as u64) as *const u32) }
    }

    /// Write a 32-bit bus-master register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped bus-master page and 32-bit aligned.
    pub unsafe fn bm32_write(&self, offset: u32, value: u32) {
        unsafe { write_volatile((self.bm_base + offset as u64) as *mut u32, value) }
    }

    /// Read a 16-bit name-base (codec) register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped name-base page and even.
    pub unsafe fn nam16(&self, offset: u32) -> u16 {
        unsafe { read_volatile((self.nam_base + offset as u64) as *const u16) }
    }

    /// Write a 16-bit name-base (codec) register.
    ///
    /// # Safety
    ///
    /// `offset` must be within the mapped name-base page and even.
    pub unsafe fn nam16_write(&self, offset: u32, value: u16) {
        unsafe { write_volatile((self.nam_base + offset as u64) as *mut u16, value) }
    }

    /// Cold-reset the codec and wait for the bus to come back.
    ///
    /// # Safety
    ///
    /// Requires valid register windows.
    pub unsafe fn cold_reset(&self) -> bool {
        unsafe {
            // 0x04000000 is the AC97 cold-reset bit in GLOB_CNT.
            self.bm32_write(GLOB_CNT, 0x0400_0000);
            for _ in 0..100_000 {
                if self.bm32(GLOB_STA) & ST_EOBI != 0 {
                    return true;
                }
            }
            false
        }
    }

    /// Point the playback engine at a descriptor list and enable it.
    ///
    /// # Safety
    ///
    /// `bdl_phys` must address at least `entries` descriptors that stay valid
    /// until the engine is stopped.
    pub unsafe fn start_playback(&self, bdl_phys: u64, entries: u8) {
        unsafe {
            self.bm8_write(PO_CR, 0);
            self.bm32_write(PI_BDBAR, bdl_phys as u32);
            self.bm8_write(PI_LVI, entries.saturating_sub(1));
            self.bm8_write(PO_CR, 1);
        }
    }

    /// Stop the playback engine.
    ///
    /// # Safety
    ///
    /// Requires valid register windows.
    pub unsafe fn stop_playback(&self) {
        unsafe { self.bm8_write(PO_CR, 0) };
    }

    /// Whether the playback engine reported an end-of-buffer condition.
    ///
    /// # Safety
    ///
    /// Requires valid register windows.
    pub unsafe fn playback_xrun(&self) -> bool {
        unsafe { self.bm8(PI_SR) & 0x08 != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bdl_entry_masks_the_length_out_of_the_control_word() {
        let entry = BdlEntry::new(0x8000_0000, 4096);
        assert_eq!(entry.frames(), 4096);
        assert!(!entry.is_last());

        let last = BdlEntry::last(0x8000_0000, 512);
        assert_eq!(last.frames(), 512);
        assert!(last.is_last());
    }

    #[test]
    fn bdl_entry_matches_the_odin_driver_encoding() {
        // The Odin driver built entries as `{addr, chunk | IOC}`; the same
        // encoding must survive the port, because the hardware reads it.
        let addr = 0x0001_0000u32;
        let chunk = 4096u32;
        let entry = BdlEntry::last(addr, chunk);
        assert_eq!(entry.addr, addr);
        assert_eq!(entry.ctrl, chunk | IOC);
    }

    #[test]
    fn register_offsets_match_the_odin_driver() {
        // These are the constants the legacy Odin code used verbatim.
        assert_eq!(GLOB_CNT, 0x2C);
        assert_eq!(GLOB_STA, 0x30);
        assert_eq!(PI_BDBAR, 0x00);
        assert_eq!(PI_LVI, 0x05);
        assert_eq!(PI_SR, 0x06);
        assert_eq!(PO_CR, 0x1B);
        assert_eq!(CI_BDBAR, 0x10);
        assert_eq!(CI_LVI, 0x15);
        assert_eq!(CI_SR, 0x16);
        assert_eq!(CO_CR, 0x1B);
    }
}
