

use crate::allocator::traits::{Frame, FrameAllocator, MapError, MapFlags, PhysAddr, VirtAddr, VirtualMapper};

const ENTRY_COUNT: usize = 512;

const FLAG_PRESENT: u64 = 1 << 0;
const FLAG_WRITABLE: u64 = 1 << 1;
const FLAG_USER: u64 = 1 << 2;
const FLAG_NO_EXECUTE: u64 = 1 << 63;
const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct PageTableEntry(u64);

impl PageTableEntry {
    const fn unused() -> Self { Self(0) }
    fn is_present(&self) -> bool { self.0 & FLAG_PRESENT != 0 }
    fn addr(&self) -> PhysAddr { PhysAddr::new(self.0 & ADDR_MASK) }

    fn set(&mut self, addr: PhysAddr, flags: MapFlags) {
        let mut raw = addr.as_u64() & ADDR_MASK;
        raw |= FLAG_PRESENT;
        if flags.contains(MapFlags::WRITABLE) { raw |= FLAG_WRITABLE; }
        if flags.contains(MapFlags::USER_ACCESSIBLE) { raw |= FLAG_USER; }
        if flags.contains(MapFlags::NO_EXECUTE) { raw |= FLAG_NO_EXECUTE; }
        self.0 = raw;
    }

    fn clear(&mut self) { self.0 = 0; }
}

#[repr(align(4096))]
struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    unsafe fn zeroed_at(ptr: *mut PageTable) {
        for i in 0..ENTRY_COUNT {
            (*ptr).entries[i] = PageTableEntry::unused();
        }
    }
}

/// Mapper operujący na drzewie tablic stron zaczynającym się w `pml4_phys`.
pub struct PageMapper {
    pml4_phys: PhysAddr,
    phys_to_virt_offset: u64,
}

impl PageMapper {
    pub fn new(_pml4_frame: Frame, pml4_phys: PhysAddr, phys_to_virt_offset: u64) -> Self {
        Self { pml4_phys, phys_to_virt_offset }
    }

    #[inline]
    fn phys_to_table_ptr(&self, phys: PhysAddr) -> *mut PageTable {
        (phys.as_u64() + self.phys_to_virt_offset) as *mut PageTable
    }

    fn table_indices(virt: VirtAddr) -> [usize; 4] {
        let addr = virt.as_u64();
        [
            ((addr >> 39) & 0x1ff) as usize,
            ((addr >> 30) & 0x1ff) as usize,
            ((addr >> 21) & 0x1ff) as usize,
            ((addr >> 12) & 0x1ff) as usize,
        ]
    }

    unsafe fn next_table_or_create(
        &mut self,
        table: *mut PageTable,
        index: usize,
        frame_alloc: &mut dyn FrameAllocator,
    ) -> Result<*mut PageTable, MapError> {
        let entry = &mut (*table).entries[index];

        if entry.is_present() {
            return Ok(self.phys_to_table_ptr(entry.addr()));
        }

        let frame = frame_alloc.allocate_frame().ok_or(MapError::FrameAllocationFailed)?;
        let page_size = crate::allocator::config::PAGE_SIZE as u64;
        let phys = PhysAddr::new((frame.0 as u64) * page_size);
        let new_table_ptr = self.phys_to_table_ptr(phys);
        PageTable::zeroed_at(new_table_ptr);

        entry.set(phys, MapFlags::PRESENT | MapFlags::WRITABLE);
        Ok(new_table_ptr)
    }

    unsafe fn next_table(&self, table: *mut PageTable, index: usize) -> Option<*mut PageTable> {
        let entry = (*table).entries[index];
        if entry.is_present() {
            Some(self.phys_to_table_ptr(entry.addr()))
        } else {
            None
        }
    }
}

impl VirtualMapper for PageMapper {
    unsafe fn map(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: MapFlags,
        frame_alloc: &mut dyn FrameAllocator,
    ) -> Result<(), MapError> {
        let [i4, i3, i2, i1] = Self::table_indices(virt);
        let pml4 = self.phys_to_table_ptr(self.pml4_phys);

        let pdpt = self.next_table_or_create(pml4, i4, frame_alloc)?;
        let pd = self.next_table_or_create(pdpt, i3, frame_alloc)?;
        let pt = self.next_table_or_create(pd, i2, frame_alloc)?;

        let entry = &mut (*pt).entries[i1];
        if entry.is_present() {
            return Err(MapError::PageAlreadyMapped);
        }
        entry.set(phys, flags);

        core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack, preserves_flags));
        Ok(())
    }

    unsafe fn unmap(&mut self, virt: VirtAddr) -> Result<PhysAddr, MapError> {
        let [i4, i3, i2, i1] = Self::table_indices(virt);
        let pml4 = self.phys_to_table_ptr(self.pml4_phys);

        let pdpt = self.next_table(pml4, i4).ok_or(MapError::PageNotMapped)?;
        let pd = self.next_table(pdpt, i3).ok_or(MapError::PageNotMapped)?;
        let pt = self.next_table(pd, i2).ok_or(MapError::PageNotMapped)?;

        let entry = &mut (*pt).entries[i1];
        if !entry.is_present() {
            return Err(MapError::PageNotMapped);
        }
        let phys = entry.addr();
        entry.clear();

        core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack, preserves_flags));
        Ok(phys)
    }

    fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        unsafe {
            let [i4, i3, i2, i1] = Self::table_indices(virt);
            let pml4 = self.phys_to_table_ptr(self.pml4_phys);

            let pdpt = self.next_table(pml4, i4)?;
            let pd = self.next_table(pdpt, i3)?;
            let pt = self.next_table(pd, i2)?;

            let entry = (*pt).entries[i1];
            if entry.is_present() {
                let offset = virt.as_u64() & 0xfff;
                Some(PhysAddr::new(entry.addr().as_u64() + offset))
            } else {
                None
            }
        }
    }
}