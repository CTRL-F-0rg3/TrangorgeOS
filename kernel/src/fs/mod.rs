pub mod driver;
pub mod mbr;

use crate::testing::TestResult;

/// Inicjalizacja podsystemu plików: wykrywa dysk ATA/IDE i parsuje MBR.
pub fn init() {
    driver::init();
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

    Ok("ATA + MBR OK")
}
