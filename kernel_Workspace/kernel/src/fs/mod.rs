pub mod driver;
pub mod mbr;
pub mod tangfs;
pub mod ext4;
pub mod fat32;
pub mod vfs;
pub mod uspace;
pub mod bootmenu;
pub mod session;
pub mod shellfs;
use crate::fs::driver::block::BlockDevice;
use crate::fs::tangfs::tfs::{format, read_superblock, Result};
use crate::testing::TestResult;

/// Bring up the storage stack: probe the ATA bus, read the partition table,
/// then mount whatever filesystem the data disk carries.
///
/// `mount_all` runs *after* `ensure_formatted` in `self_test` rather than here.
/// That ordering is deliberate: a freshly created `data.img` has no superblock
/// at all, and `tangfs::read_superblock` would reject it. `mount_all` treats
/// "not Tangfs" as "try the next filesystem", so probing before the format would
/// miss the only filesystem present.
pub fn init() {
    driver::init();
    mbr::init();
}

/// Mount the first filesystem found on any registered device.
///
/// Separate from [`init`] because it needs a formatted medium first. Call it
/// once the data disk carries a filesystem; calling it earlier is harmless but
/// finds nothing.
pub fn mount_root() {
    if vfs::root().is_none() {
        vfs::mount_all();
    }
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

    // The data disk may be raw/unformatted (e.g. a freshly created data.img),
    // so make sure it carries a TFS superblock before touching the tree.
    if ensure_formatted(data).is_err() {
        return Err("tfs format failed");
    }

    // The medium is formatted now, so the VFS has something to mount. Without
    // this, `vfs::root()` stays `None` and every path lookup falls back to
    // re-resolving against the raw device.
    mount_root();

    if vfs::root().is_none() {
        return Err("no filesystem mounted after format");
    }

    // Ensure the base configuration files exist, whether this is a fresh
    // install or an already-formatted disk.
    if seed_defaults(data).is_err() {
        return Err("seeding base config failed");
    }

    // The base configuration files must be present after a fresh install.
    if tangfs::tfs::read_file(data, tangfs::tfs::ROOT_DIR, "config.tcfg").is_err() {
        return Err("missing base config (config.tcfg)");
    }

    if tangfs::tfs::write_file(data, tangfs::tfs::ROOT_DIR, "hello.txt", b"Hello from TFS on disk!").is_err() {
        return Err("tfs write failed");
    }

    if tangfs::tfs::write_file(data, tangfs::tfs::ROOT_DIR, "note.txt", b"second file").is_err() {
        return Err("tfs write #2 failed");
    }

    match tangfs::tfs::read_file(data, tangfs::tfs::ROOT_DIR, "hello.txt") {
        Ok(d) if d == b"Hello from TFS on disk!" => {}
        _ => return Err("tfs readback mismatch"),
    }

    if tangfs::tfs::mkdir(data, tangfs::tfs::ROOT_DIR, "docs").is_err() {
        return Err("tfs mkdir failed");
    }

    let docs = match tangfs::tfs::find_dir(data, tangfs::tfs::ROOT_DIR, "docs") {
        Ok(d) => d,
        Err(_) => return Err("tfs find_dir failed"),
    };

    if tangfs::tfs::write_file(data, docs, "readme.txt", b"inside a folder").is_err() {
        return Err("tfs write in dir failed");
    }

    match tangfs::tfs::read_file(data, docs, "readme.txt") {
        Ok(d) if d == b"inside a folder" => {}
        _ => return Err("tfs dir readback mismatch"),
    }

    if tangfs::tfs::remove(data, docs, "readme.txt").is_err() {
        return Err("tfs rm in dir failed");
    }

    if tangfs::tfs::remove(data, tangfs::tfs::ROOT_DIR, "docs").is_err() {
        return Err("tfs rmdir failed");
    }

    if tangfs::tfs::remove(data, tangfs::tfs::ROOT_DIR, "note.txt").is_err() {
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

/// File extension used by TrangorgeOS system configuration files. These files
/// describe "how the system should work" and are always present on a fresh
/// install, persisted on the data disk.
pub const CFG_EXT: &str = "tcfg";

/// Base configuration files seeded onto a freshly formatted disk.
const DEFAULT_CONFIGS: &[(&str, &str)] = &[
    (
        "autostart.tcfg",
        "# TrangorgeOS autostart configuration\n\
         # One command per line; lines starting with '#' are comments.\n\
         allde\n",
    ),
    (
        "config.tcfg",
        "# TrangorgeOS system configuration\n\
         hostname=TrangorgeOS\n\
         timezone=UTC\n\
         keyboard=us\n\
         resolution=1920x1080\n",
    ),
    (
        "drivers.tcfg",
        "# TrangorgeOS driver autoload list\n\
         # One driver name per line.\n\
         ata\n\
         xhci\n\
         virtio_net\n",
    ),
];

/// Writes the base system configuration files (`autostart.tcfg`, `config.tcfg`,
/// `drivers.tcfg`) to the root of the data disk. Only missing files are
/// created, so this is safe to run on every boot: it neither overwrites user
/// edits nor leaks data blocks on an already-populated disk.
pub fn seed_defaults(dev: &dyn BlockDevice) -> Result<()> {
    for (name, contents) in DEFAULT_CONFIGS {
        if tangfs::tfs::read_file(dev, tangfs::tfs::ROOT_DIR, name).is_err() {
            tangfs::tfs::write_file(dev, tangfs::tfs::ROOT_DIR, name, contents.as_bytes())?;
        }
    }
    Ok(())
}

/// Makes sure the given block device carries a valid TFS superblock,
/// formatting it when it is still raw (e.g. a freshly created data.img).
pub fn ensure_formatted(dev: &dyn BlockDevice) -> Result<()> {
    match read_superblock(dev) {
        Ok(_) => Ok(()),
        Err(_) => {
            crate::println!("[fs] formatting disk with TFS...");
            format(dev)
        }
    }
}
