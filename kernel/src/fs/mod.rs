pub mod driver;
pub mod mbr;
pub mod tfs;

use crate::testing::TestResult;

/// Inicjalizacja podsystemu plików: wykrywa dysk ATA/IDE i parsuje MBR.
pub fn init() {
    driver::init();
    mbr::init();
}

/// Samotest FS: wykrycie dysku, odczyt sektora 0 i weryfikacja MBR.
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

    // --- Roundtrip TFS na dysku danych (slave) ---
    let data = match root_device() {
        Some(d) => d,
        None => return Err("no data disk"),
    };

    if tfs::format(data).is_err() {
        return Err("tfs format failed");
    }

    if tfs::write_file(data, "hello.txt", b"Hello from TFS on disk!").is_err() {
        return Err("tfs write failed");
    }

    if tfs::write_file(data, "note.txt", b"second file").is_err() {
        return Err("tfs write #2 failed");
    }

    match tfs::read_file(data, "hello.txt") {
        Ok(d) if d == b"Hello from TFS on disk!" => {}
        _ => return Err("tfs readback mismatch"),
    }

    match tfs::entries(data) {
        Ok(v) if v.len() == 2 => {}
        _ => return Err("tfs entry count mismatch"),
    }

    if tfs::remove_file(data, "note.txt").is_err() {
        return Err("tfs rm failed");
    }

    Ok("ATA + MBR + TFS (format/write/read/rm) OK")
}

/// Zwraca pierwszą dostępną partycję lub cały dysk — dla terminala i TFS.
pub fn root_device() -> Option<&'static dyn driver::block::BlockDevice> {
    // Preferujemy partycję (jeśli jest), inaczej cały dysk.
    if driver::registry::count() > 1 {
        driver::registry::get(1)
    } else {
        driver::registry::first()
    }
}

