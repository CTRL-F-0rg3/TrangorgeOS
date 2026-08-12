pub mod bitmap;
pub mod boot_bridge;
pub mod buddy;

use crate::allocator::config::MAX_SUPPORTED_FRAMES;
use bitmap::BitmapFrameAllocator;
use bootloader::bootinfo::MemoryMap;
use spin::Mutex;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

const BITMAP_WORDS: usize = MAX_SUPPORTED_FRAMES / 64;

static mut BITMAP_STORAGE: [u64; BITMAP_WORDS] = [0; BITMAP_WORDS];

pub static FRAME_ALLOCATOR: Mutex<Option<BitmapFrameAllocator<'static>>> = Mutex::new(None);

pub fn init(memory_map: &'static MemoryMap) {
    let total_frames = boot_bridge::total_frames(memory_map).min(MAX_SUPPORTED_FRAMES);
    let storage: &'static mut [u64] = unsafe { &mut *core::ptr::addr_of_mut!(BITMAP_STORAGE) };
    let mut allocator = BitmapFrameAllocator::new(storage, total_frames);
    boot_bridge::populate(&mut allocator, memory_map);
    *FRAME_ALLOCATOR.lock() = Some(allocator);
}

pub fn allocate_frame() -> Option<PhysFrame<Size4KiB>> {
    let mut guard = FRAME_ALLOCATOR.lock();
    let allocator = guard.as_mut()?;
    let idx = allocator.allocate_frame()?;
    Some(PhysFrame::containing_address(PhysAddr::new(
        BitmapFrameAllocator::frame_to_addr(idx),
    )))
}

pub fn deallocate_frame(frame: PhysFrame<Size4KiB>) {
    if let Some(allocator) = FRAME_ALLOCATOR.lock().as_mut() {
        let idx = BitmapFrameAllocator::addr_to_frame(frame.start_address().as_u64());
        allocator.deallocate_frame(idx);
    }
}

pub struct GlobalFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        allocate_frame()
    }
}
