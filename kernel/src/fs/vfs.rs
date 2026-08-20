use crate::fs::ext4::Ext4;
use crate::fs::fat32::Fat32;
use crate::fs::driver::registry;
use alloc::vec::Vec;

pub struct FsEntry {
    pub name: Vec<u8>,
    pub is_dir: bool,
    pub size: u64,
}

pub enum Mounted {
    Ext4(Ext4),
    Fat32(Fat32),
}

impl Mounted {
    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Option<usize> {
        match self {
            Mounted::Ext4(f) => f.read_path(path, buf).ok(),
            Mounted::Fat32(f) => f.read_path(path, buf).ok(),
        }
    }

    pub fn list_path(&self, path: &str) -> Option<Vec<FsEntry>> {
        match self {
            Mounted::Ext4(f) => f.list_path(path).ok().map(|v| {
                v.into_iter().map(|e| FsEntry {
                    name: e.name,
                    is_dir: e.ftype == 2,
                    size: 0,
                }).collect()
            }),
            Mounted::Fat32(f) => f.list_path(path).ok().map(|v| {
                v.into_iter().map(|e| FsEntry {
                    name: e.name,
                    is_dir: e.is_dir,
                    size: e.size,
                }).collect()
            }),
        }
    }
}

static mut ROOT_FS: Option<Mounted> = None;
pub fn register_tangfs() {
    use alloc::boxed::Box;
    
    FILESYSTEMS.lock().push(Box::new(|device| {
        tangfs::TangFs::mount(device).map(|fs| Box::new(fs) as Box<dyn FileSystem>)
    }));
}
pub fn mount_all() {
    let n = registry::count();

    for i in 0..n {
        if let Some(d) = registry::get(i) {
            if let Ok(f) = Ext4::mount(d) {
                unsafe { ROOT_FS = Some(Mounted::Ext4(f)) };
                return;
            }

            if let Ok(f) = Fat32::mount(d) {
                unsafe { ROOT_FS = Some(Mounted::Fat32(f)) };
                return;
            }
        }
    }
}

pub fn root() -> Option<&'static Mounted> {
    unsafe { ROOT_FS.as_ref() }
}






// extern "C" fn cmd_ls(arg: *mut u8, len: u32) -> i32 {
//     if let Some(fs) = vfs::root() {
//         if let Some(entries) = fs.list_path("/") {
//             for e in entries {
//                 let kind = if e.is_dir { b"d" } else { b"-" };
//                 kprintf(b"%s %s (%d B)\n\0", kind, e.name.as_ptr(), e.size);
//             }
//         }
//     }

//     0
// }


