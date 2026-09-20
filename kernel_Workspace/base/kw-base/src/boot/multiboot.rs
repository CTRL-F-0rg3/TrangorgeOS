use kstd_base::PhysAddr;

pub const MULTIBOOT2_MAGIC: u32 = 0x36d76289;

#[repr(C)]
pub struct Multiboot2Info {
    pub total_size: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct Multiboot2Tag {
    pub tag_type: u32,
    pub size: u32,
}

pub const TAG_TYPE_END: u32 = 0;
pub const TAG_TYPE_CMDLINE: u32 = 1;
pub const TAG_TYPE_BOOTLOADER: u32 = 2;
pub const TAG_TYPE_MODULE: u32 = 3;
pub const TAG_TYPE_BASIC_MEMINFO: u32 = 4;
pub const TAG_TYPE_BOOTDEV: u32 = 5;
pub const TAG_TYPE_MMAP: u32 = 6;
pub const TAG_TYPE_FRAMEBUFFER: u32 = 8;

#[repr(C)]
pub struct MmapTag {
    pub tag: Multiboot2Tag,
    pub entry_size: u32,
    pub entry_version: u32,
}

#[repr(C)]
pub struct MmapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    pub reserved: u32,
}

pub struct BootInfo {
    pub ptr: *const Multiboot2Info,
}

unsafe impl Send for BootInfo {}
unsafe impl Sync for BootInfo {}

impl BootInfo {
    pub unsafe fn from_ptr(ptr: *const u8) -> Self {
        Self { ptr: ptr as *const Multiboot2Info }
    }

    pub fn first_tag(&self) -> *const Multiboot2Tag {
        unsafe { (self.ptr as *const u8).add(8) as *const Multiboot2Tag }
    }

    pub fn next_tag(tag: *const Multiboot2Tag) -> *const Multiboot2Tag {
        unsafe {
            let addr = tag as usize;
            let next = (addr + (*tag).size as usize + 7) & !7;
            next as *const Multiboot2Tag
        }
    }

    pub fn find_tag(&self, tag_type: u32) -> *const Multiboot2Tag {
        let mut tag = self.first_tag();
        unsafe {
            while (*tag).tag_type != TAG_TYPE_END {
                if (*tag).tag_type == tag_type { return tag; }
                tag = Self::next_tag(tag);
            }
        }
        core::ptr::null()
    }
}