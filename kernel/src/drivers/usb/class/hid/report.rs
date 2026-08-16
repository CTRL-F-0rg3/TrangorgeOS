pub struct BootReport {
    pub shift: bool,
    pub keys: [u8; 6],
}

pub fn parse_boot_keyboard(report: &[u8]) -> Option<BootReport> {
    if report.len() < 8 {
        return None;
    }

    let mut keys = [0u8; 6];
    keys.copy_from_slice(&report[2..8]);

    Some(BootReport {
        shift: report[0] & 0x22 != 0,
        keys,
    })
}