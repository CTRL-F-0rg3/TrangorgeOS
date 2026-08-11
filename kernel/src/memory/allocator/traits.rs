
use core::ops::{Add, Sub, BitOr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(u64);

impl PhysAddr {
    #[inline] pub const fn new(addr: u64) -> Self { Self(addr) }
    #[inline] pub const fn as_u64(self) -> u64 { self.0 }
    #[inline] pub const fn as_usize(self) -> usize { self.0 as usize }
}

impl VirtAddr {
    #[inline] pub const fn new(addr: u64) -> Self { Self(addr) }
    #[inline] pub const fn as_u64(self) -> u64 { self.0 }
    #[inline] pub const fn as_usize(self) -> usize { self.0 as usize }
    #[inline] pub const fn as_mut_ptr<T>(self) -> *mut T { self.0 as *mut T }
}

impl Add<u64> for PhysAddr { type Output = PhysAddr; fn add(self, rhs: u64) -> PhysAddr { PhysAddr(self.0 + rhs) } }
impl Sub<u64> for PhysAddr { type Output = PhysAddr; fn sub(self, rhs: u64) -> PhysAddr { PhysAddr(self.0 - rhs) } }
impl Add<u64> for VirtAddr { type Output = VirtAddr; fn add(self, rhs: u64) -> VirtAddr { VirtAddr(self.0 + rhs) } }
impl Sub<u64> for VirtAddr { type Output = VirtAddr; fn sub(self, rhs: u64) -> VirtAddr { VirtAddr(self.0 - rhs) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame(pub usize);

impl Frame {
    pub fn containing_address(addr: PhysAddr, page_size: usize) -> Self {
        Frame(addr.as_usize() / page_size)
    }
    pub fn start_address(self, page_size: usize) -> PhysAddr {
        PhysAddr::new((self.0 * page_size) as u64)
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
    fn total_frames(&self) -> usize;
    fn free_frames(&self) -> usize;
    fn used_frames(&self) -> usize { self.total_frames() - self.free_frames() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapFlags(u32);

impl MapFlags {
    pub const PRESENT: MapFlags = MapFlags(1 << 0);
    pub const WRITABLE: MapFlags = MapFlags(1 << 1);
    pub const USER_ACCESSIBLE: MapFlags = MapFlags(1 << 2);
    pub const NO_EXECUTE: MapFlags = MapFlags(1 << 3);

    pub const fn empty() -> Self { MapFlags(0) }
    pub const fn contains(self, other: MapFlags) -> bool { (self.0 & other.0) == other.0 }
    pub const fn union(self, other: MapFlags) -> Self { MapFlags(self.0 | other.0) }
}

impl BitOr for MapFlags {
    type Output = MapFlags;
    fn bitor(self, rhs: MapFlags) -> MapFlags { self.union(rhs) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    FrameAllocationFailed,
    PageAlreadyMapped,
    PageNotMapped,
    HugePageNotSupported,
}

pub trait VirtualMapper {
    unsafe fn map(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: MapFlags,
        frame_alloc: &mut dyn FrameAllocator,
    ) -> Result<(), MapError>;

    unsafe fn unmap(&mut self, virt: VirtAddr) -> Result<PhysAddr, MapError>;

    fn translate(&self, virt: VirtAddr) -> Option<PhysAddr>;
}