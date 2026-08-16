pub mod ata_pio;
pub mod block;
pub mod lock;
pub mod port;
pub mod registry;

pub fn init() {
    if ata_pio::probe() {
        registry::register(&ata_pio::ATA0);
    }
}

pub fn self_test() -> bool {
    let dev = match registry::first() {
        Some(d) => d,
        None => return false,
    };

    let mut buf = [0u8; 512];

    if dev.read_block(0, &mut buf).is_err() {
        return false;
    }

    buf[510] == 0x55 && buf[511] == 0xAA
}