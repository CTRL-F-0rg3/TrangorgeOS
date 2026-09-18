use super::elf;
use crate::trampoline_rings as tr;

const USER_STACK_TOP: u64 = 0x7FFF_0000_0000;

pub fn spawn_init() -> Result<(), &'static str> {
    let mut buf = [0u8; 64 * 1024];

    let n = crate::fs::vfs::read_path("/bin/init.elf", &mut buf)
        .ok_or("init.elf not found in fs")?;

    let loaded = elf::load(&buf[..n])?;

    let stack_phys = crate::mm::phys::alloc_frames(4)
        .ok_or("no stack mem")?;

    let prot = crate::mm::space::PROT_READ
             | crate::mm::space::PROT_WRITE
             | crate::mm::space::PROT_USER;

    if !loaded.aspace.map_phys(USER_STACK_TOP - 0x4000,
                               stack_phys, 0x4000, prot) {
        return Err("stack map fail");
    }

    tr::add_world(tr::RING_USER,
                  loaded.aspace.cr3(),
                  loaded.entry,
                  USER_STACK_TOP,
                  0)
        .ok_or("world table full")?;

    extern "C" { fn kprintf(fmt: *const u8, ...); }
    unsafe {
        kprintf(b"[boot] userspace init spawned from /bin/init.elf (last stage)\n\0".as_ptr());
    }

    Ok(())
}