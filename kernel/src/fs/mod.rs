pub mod driver;
pub mod mbr;
pub mod tfs;
use crate::fs::driver::block::BlockDevice;
use crate::fs::driver::registry;
use crate::fs::tfs::{format, read_superblock, Result};
use crate::testing::TestResult;

pub fn init() {
    driver::init();
    mbr::init();
}

pub fn self_test() -> TestResult {
    if driver::registry::count() == 0 {
        driver::init();
    }

    let count = driver::registry::count();

    if count == 0 {
        crate::println!("[fs] no ATA/IDE block device detected");
        return Err("no ATA/IDE disk");
    }

    let dev = match driver::registry::first() {
        Some(d) => d,
        None => return Err("block-device registry empty"),
    };

    crate::println!(
        "[fs] device: name={} block_size={} blocks={}",
        dev.name(),
        dev.block_size(),
        dev.block_count()
    );

    let mut buf = [0u8; 512];

    if dev.read_block(0, &mut buf).is_err() {
        return Err("read sector 0 failed");
    }

    if buf[510] != 0x55 || buf[511] != 0xAA {
        return Err("no MBR signature (0x55AA)");
    }

    let parts = mbr::probe_disk(dev);

    crate::println!("[fs] MBR valid, {} partition(s)", parts);

    let data = match root_device() {
        Some(d) => d,
        None => return Err("no data disk"),
    };

    pub fn ensure_formatted(dev: &dyn BlockDevice) -> Result<()> {
        match read_superblock(dev) {
            Ok(_) => Ok(()),
            Err(_) => {
                crate::println!("[fs] formatting disk with TFS...");
                format(dev)
            }
        }
    }

    if tfs::write_file(data, tfs::ROOT_DIR, "hello.txt", b"Hello from TFS on disk!").is_err() {
        return Err("tfs write failed");
    }

    if tfs::write_file(data, tfs::ROOT_DIR, "note.txt", b"second file").is_err() {
        return Err("tfs write #2 failed");
    }

    match tfs::read_file(data, tfs::ROOT_DIR, "hello.txt") {
        Ok(d) if d == b"Hello from TFS on disk!" => {}
        _ => return Err("tfs readback mismatch"),
    }

    if tfs::mkdir(data, tfs::ROOT_DIR, "docs").is_err() {
        return Err("tfs mkdir failed");
    }

    let docs = match tfs::find_dir(data, tfs::ROOT_DIR, "docs") {
        Ok(d) => d,
        Err(_) => return Err("tfs find_dir failed"),
    };

    if tfs::write_file(data, docs, "readme.txt", b"inside a folder").is_err() {
        return Err("tfs write in dir failed");
    }

    match tfs::read_file(data, docs, "readme.txt") {
        Ok(d) if d == b"inside a folder" => {}
        _ => return Err("tfs dir readback mismatch"),
    }

    if tfs::remove(data, docs, "readme.txt").is_err() {
        return Err("tfs rm in dir failed");
    }

    if tfs::remove(data, tfs::ROOT_DIR, "docs").is_err() {
        return Err("tfs rmdir failed");
    }

    if tfs::remove(data, tfs::ROOT_DIR, "note.txt").is_err() {
        return Err("tfs rm failed");
    }

    Ok("ATA + MBR + TFS (format/write/read/mkdir/rm) OK")
}

pub fn root_device() -> Option<&'static dyn driver::block::BlockDevice> {
    if driver::registry::count() > 1 {
        driver::registry::get(1)
    } else {
        driver::registry::first()
    }
}
