use crate::fs::disc::read_mbr_partitions;
use crate::fs::driver::REGISTRY;
use crate::fs::share::FilesystemKind;
use crate::println;

pub fn init() {
    crate::fs::driver::init::init();

    let mut registry = REGISTRY.lock();
    let device_count = registry.len();

    for index in 0..device_count {
        let Some(device) = registry.get(index) else {
            continue;
        };

        let partitions = match read_mbr_partitions(device) {
            Ok(p) => p,
            Err(_) => continue,
        };

        for partition in partitions {
            match partition.fs_hint {
                FilesystemKind::Fat32 => {
                    println!(
                        "fs: device {} partition at LBA {} hinted as FAT32",
                        index, partition.start_lba
                    );
                }
                FilesystemKind::Ext4 => {
                    println!(
                        "fs: device {} partition at LBA {} hinted as ext4 (mount unsupported)",
                        index, partition.start_lba
                    );
                }
                FilesystemKind::Unknown => {}
            }
        }
    }
}
