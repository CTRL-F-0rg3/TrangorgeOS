//! FAT32 with write support.
//!
//! Ported from `fs/fat/` in the Linux reference tree, with the on-disk constants
//! taken from `include/uapi/linux/msdos_fs.h` rather than from a datasheet.
//!
//! # What changed against the read-only driver
//!
//! The previous `fat32/mod.rs` could mount, resolve and read. It could not write,
//! and two of its omissions were not merely missing features but wrong answers:
//!
//! * **`mount` accepted anything with an MBR signature.** It never checked the
//!   filesystem type, so mounting an ext4 volume produced a `Fat32` that appeared
//!   to work and returned another filesystem's data as directory entries.
//! * **Nothing validated `root_cluster`.** Zero or one — the reserved clusters —
//!   walked into the reserved area and returned whatever was there as names.
//!
//! # The write path, and why the order is the design
//!
//! 1. **Allocate clusters** through the FAT. A crash between steps 2 and 3 leaves
//!    orphaned clusters, which the next mount can reclaim.
//! 2. **Write the data.**
//! 3. **Write the directory entry** last, with the cluster chain and the size.
//!
//! A directory entry is the only thing that makes a file visible. Until step 3
//! the data is unreachable and harmless; after it, reachable and consistent.
//! Reversing 2 and 3 yields a listed file with the wrong size and no data behind
//! it, which is worse than a missing file.
//!
//! # What is not here
//!
//! Long-filename *writing*. New entries carry their 8.3 short name, which every
//! FAT32 reader displays. LFN writing needs a per-entry checksum in the sequence
//! field, and a wrong checksum makes the long-name slots invisible *and* orphans
//! the short name that follows them.

#![allow(dead_code)]

use crate::fs::driver::block::{BlockDevice, DriverError};
use alloc::vec;
use alloc::vec::Vec;

/// Sector size in bytes. Fixed by the format: FAT cannot describe another.
pub const SECTOR_SIZE: usize = 512;

/// Directory entries per sector.
pub const DIR_PER_SECTOR: usize = SECTOR_SIZE / 32;

/// `FAT32_EOC`: end of a cluster chain.
pub const EOC: u32 = 0x0FFF_FFFF;
/// A cluster the media has marked bad.
pub const BAD_CLUSTER: u32 = 0x0FFF_FFF7;
/// A free cluster: the FAT entry is zero.
pub const FREE: u32 = 0;
/// Clusters 0 and 1 are reserved; data starts at 2.
pub const FIRST_DATA_CLUSTER: u32 = 2;

/// The FSINFO lead signature, "RRaA".
pub const FSINFO_SIG1: u32 = 0x4161_5252;
/// The FSINFO struct signature, "rrAa".
pub const FSINFO_SIG2: u32 = 0x6141_7272;

/// A short-name first byte marking the entry deleted.
pub const DELETED_MARK: u8 = 0xE5;
/// A short-name first byte of `0x00` ends the directory.
pub const END_OF_DIR: u8 = 0x00;
/// `0x05` starts a volume label, which is not a file.
pub const VOLUME_LABEL: u8 = 0x05;
/// `0x0F` is a long-name entry, not a file.
pub const LFN_MARK: u8 = 0x0F;

/// Attribute bit: read-only.
pub const ATTR_READ_ONLY: u8 = 0x01;
/// Attribute bit: hidden.
pub const ATTR_HIDDEN: u8 = 0x02;
/// Attribute bit: system.
pub const ATTR_SYSTEM: u8 = 0x04;
/// Attribute bit: volume label.
pub const ATTR_VOLUME: u8 = 0x08;
/// Attribute bit: a directory.
pub const ATTR_DIRECTORY: u8 = 0x10;
/// Attribute bit: archived.
pub const ATTR_ARCHIVE: u8 = 0x20;

/// Why a FAT32 operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatError {
    /// The media reported an I/O error.
    Io,
    /// The volume is not FAT32, or its BPB is self-inconsistent.
    NotFat32,
    /// No such file or directory.
    NotFound,
    /// A name already exists.
    Exists,
    /// A path component is a file, not a directory.
    NotDir,
    /// The target is a directory, and the operation needs a file.
    IsDir,
    /// The directory being removed is not empty.
    NotEmpty,
    /// No free cluster: the volume is full.
    NoSpace,
    /// A name is empty, too long, or has characters FAT cannot store.
    BadName,
    /// A cluster number is outside the data region, or is a bad-cluster marker.
    BadCluster,
}

impl From<DriverError> for FatError {
    #[inline]
    fn from(e: DriverError) -> Self {
        match e {
            DriverError::InvalidBlock | DriverError::InvalidLength => FatError::BadCluster,
            _ => FatError::Io,
        }
    }
}

/// The BIOS Parameter Block, as this driver uses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bpb {
    /// Bytes per logical sector.
    pub bytes_per_sector: u16,
    /// Sectors per cluster.
    pub sectors_per_cluster: u8,
    /// Reserved sectors before the first FAT.
    pub reserved_sectors: u16,
    /// How many FATs there are. Two is the usual number.
    pub num_fats: u8,
    /// Sectors occupied by one FAT.
    pub fat_sectors: u32,
    /// The root directory's first cluster, from the FAT32 BPB.
    pub root_cluster: u32,
    /// The FSINFO sector number, relative to the volume start.
    pub fsinfo_sector: u16,
    /// The volume serial number.
    pub volume_id: u32,
    /// The volume label, space-padded.
    pub volume_label: [u8; 11],
    /// The filesystem type string, space-padded.
    pub fs_type: [u8; 8],
}

impl Bpb {
    /// Parse and validate a boot sector.
    ///
    /// The validation is deliberately strict. A volume that is not FAT32 usually
    /// has zero or non-power-of-two sectors-per-cluster, a FAT length of zero, or
    /// a root cluster inside the reserved area — and each of those produces a
    /// driver that mounts, reads plausible-looking entries, and hands back
    /// another filesystem's data. Refusing here is the last point where the error
    /// can still name the real cause.
    pub fn parse(boot: &[u8]) -> Result<Self, FatError> {
        if boot.len() < SECTOR_SIZE {
            return Err(FatError::NotFat32);
        }
        if boot[510] != 0x55 || boot[511] != 0xAA {
            return Err(FatError::NotFat32);
        }

        let bytes_per_sector = u16::from_le_bytes([boot[11], boot[12]]);
        let sectors_per_cluster = boot[13];
        let reserved_sectors = u16::from_le_bytes([boot[14], boot[15]]);
        let num_fats = boot[16];
        // Offset 36 is FAT32's fat_length. The 16-bit field at 22 is FAT16's and
        // reads as zero on a real FAT32 volume, so using *that* one is how an
        // NTFS or FAT16 volume gets misidentified as FAT32.
        let fat_sectors = u32::from_le_bytes([boot[36], boot[37], boot[38], boot[39]]);
        let root_cluster = u32::from_le_bytes([boot[44], boot[45], boot[46], boot[47]]);
        let fsinfo_sector = u16::from_le_bytes([boot[48], boot[49]]);

        if bytes_per_sector as usize != SECTOR_SIZE {
            return Err(FatError::NotFat32);
        }
        if sectors_per_cluster == 0 || !sectors_per_cluster.is_power_of_two() {
            return Err(FatError::NotFat32);
        }
        // A root cluster below 2 is in the reserved area; the top of the range is
        // where the bad and EOF markers live.
        if !(FIRST_DATA_CLUSTER..=0x0FFF_FFF5).contains(&root_cluster) {
            return Err(FatError::NotFat32);
        }
        if num_fats == 0 || fat_sectors == 0 || reserved_sectors == 0 || fsinfo_sector == 0 {
            return Err(FatError::NotFat32);
        }

        let mut volume_label = [0u8; 11];
        volume_label.copy_from_slice(&boot[71..82]);
        let mut fs_type = [0u8; 8];
        fs_type.copy_from_slice(&boot[82..90]);

        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            fat_sectors,
            root_cluster,
            fsinfo_sector,
            volume_id: u32::from_le_bytes([boot[67], boot[68], boot[69], boot[70]]),
            volume_label,
            fs_type,
        })
    }

    /// Bytes one cluster holds.
    #[inline]
    pub const fn cluster_bytes(&self) -> u64 {
        self.bytes_per_sector as u64 * self.sectors_per_cluster as u64
    }

    /// The sector holding FAT entry `cluster`.
    ///
    /// Entries are four bytes each and two share a sector, so a cluster number
    /// has to be halved before it becomes a sector index. Off by a factor of two
    /// here and the driver reads the wrong half of the table and follows a
    /// plausible-looking chain into unrelated data.
    #[inline]
    pub const fn fat_sector_of(&self, cluster: u32) -> u64 {
        self.reserved_sectors as u64
            + (cluster as u64 / 2) * 4 / self.bytes_per_sector as u64
    }

    /// The sector holding the first byte of `cluster`'s data.
    #[inline]
    pub const fn data_sector_of(&self, cluster: u32) -> u64 {
        self.reserved_sectors as u64
            + self.num_fats as u64 * self.fat_sectors as u64
            + (cluster as u64 - FIRST_DATA_CLUSTER as u64) * self.sectors_per_cluster as u64
    }

    /// How many data clusters the volume has.
    ///
    /// This bounds every cluster number the driver accepts. A chain running past
    /// it is corrupt, and following it reads whatever follows the volume — on a
    /// USB stick, another device's partition table.
    pub fn cluster_count(&self, total_sectors: u32) -> u32 {
        let meta = self.reserved_sectors as u64
            + self.num_fats as u64 * self.fat_sectors as u64;
        if (total_sectors as u64) <= meta {
            return 0;
        }
        let clusters = (total_sectors as u64 - meta) / self.sectors_per_cluster as u64;
        // The count includes the two reserved entries, so the last usable
        // cluster number is two less than the count.
        (clusters + 2).min(0x0FFF_FFF6) as u32
    }
}

/// One 32-byte directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirEntry {
    /// The 8.3 name, space-padded, as it sits on disk.
    pub name: [u8; 11],
    /// The attribute byte.
    pub attr: u8,
    /// File size in bytes.
    pub size: u32,
    /// The first cluster, high 16 bits.
    pub cluster_high: u16,
    /// The first cluster, low 16 bits.
    pub cluster_low: u16,
}

impl DirEntry {
    /// Is this slot free — never used, or explicitly deleted?
    ///
    /// The two cases differ: `0x00` ends the directory, `0xE5` leaves a hole a
    /// later entry may reuse. Treating `0xE5` as end-of-directory truncates a
    /// directory at its first deleted file, hiding everything after it.
    #[inline]
    pub const fn is_free(&self) -> bool {
        self.name[0] == END_OF_DIR || self.name[0] == DELETED_MARK
    }

    /// Is this a long-name slot rather than a file entry?
    #[inline]
    pub const fn is_lfn(&self) -> bool {
        self.attr == LFN_MARK
    }

    /// Is this the volume label?
    #[inline]
    pub const fn is_volume_label(&self) -> bool {
        self.attr & ATTR_VOLUME != 0
    }

    /// Is this a directory?
    #[inline]
    pub const fn is_dir(&self) -> bool {
        self.attr & ATTR_DIRECTORY != 0
    }

    /// Is this `.` or `..`?
    #[inline]
    pub fn is_dot(&self) -> bool {
        self.name[0] == b'.' && (self.name[1] == b' ' || self.name[1] == 0)
    }

    /// The entry's first cluster, assembled from both halves.
    ///
    /// FAT32 splits it across two 16-bit fields. Taking only the low half gives a
    /// cluster number that is right for small files and wrong for every file over
    /// 64 KiB — exactly where a fragmentation bug first shows.
    #[inline]
    pub const fn first_cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) | self.cluster_low as u32
    }

    /// Set both halves of the cluster field.
    #[inline]
    pub fn set_first_cluster(&mut self, cluster: u32) {
        self.cluster_low = cluster as u16;
        self.cluster_high = (cluster >> 16) as u16;
    }

    /// Decode the 11-byte 8.3 field into a name with its dot, if any.
    ///
    /// The field is space-padded and carries a case flag after the extension, so
    /// the trailing bytes must go: read raw, a name ends in spaces and a binary
    /// byte, and never matches anything a user typed.
    pub fn short_name(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12);
        for (i, ch) in self.name[0..8].iter().enumerate() {
            // 0x05 in the first byte is really 0xE5; the format reserves it.
            if i == 0 && *ch == 0x05 {
                out.push(0xE5);
            } else {
                out.push(*ch);
            }
        }
        while out.last() == Some(&b' ') {
            out.pop();
        }
        if self.name[8] != b' ' {
            out.push(b'.');
            for ch in &self.name[8..11] {
                if *ch != b' ' {
                    out.push(*ch);
                }
            }
        }
        out
    }
}

/// Build the 8.3 field for `name`, returning `None` if it cannot be represented.
///
/// The 8.3 space is genuinely restrictive: 8 characters of stem, 3 of extension,
/// uppercase only, and no spaces or dots inside. A caller that wants long names
/// gets `None` here rather than a silently mangled name, because a mangled name
/// is how `readme.txt` becomes `README  TXT` and a user concludes the filesystem
/// ate their file.
pub fn to_83_name(name: &str) -> Option<[u8; 11]> {
    if name.is_empty() || name.len() > 12 {
        return None;
    }
    let bytes = name.as_bytes();

    // Split on the *last* dot, so `archive.tar.gz` is stem `archive.tar` — which
    // is itself invalid, and is caught below.
    let (stem, ext) = match bytes.iter().rposition(|b| *b == b'.') {
        Some(dot) if dot > 0 => (&bytes[..dot], &bytes[dot + 1..]),
        Some(_) | None => (bytes, &bytes[bytes.len()..]),
    };

    if stem.is_empty() || stem.len() > 8 || ext.len() > 3 {
        return None;
    }
    for ch in stem.iter().chain(ext.iter()) {
        let ok = ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-' | b'+' | b'@');
        if !ok {
            return None;
        }
    }

    let mut out = [b' '; 11];
    for (i, ch) in stem.iter().enumerate() {
        out[i] = ch.to_ascii_uppercase();
    }
    for (i, ch) in ext.iter().enumerate() {
        out[8 + i] = ch.to_ascii_uppercase();
    }
    Some(out)
}

/// Compare a name against a directory entry, case-insensitively.
///
/// FAT's case-folding is in the NT lowercase flags, but the common case is a
/// plain uppercase name on the disk, so an ASCII fold covers what actually
/// happens. This is also why `resolve` must fold: a USB stick written by Windows
/// holds `README.TXT`, and a lookup for `readme.txt` has to find it.
pub fn name_eq(entry: &DirEntry, query: &str) -> bool {
    let name = entry.short_name();
    if name.len() != query.len() {
        return false;
    }
    name.iter()
        .zip(query.bytes())
        .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

/// A mounted FAT32 volume.
///
/// The device is held, not borrowed per call. Every read and write path
/// (`read_file`, `create_file`, `mkdir`, `alloc_cluster`, …) needs the medium,
/// and threading it through each signature meant every caller had to keep the
/// `&'static dyn BlockDevice` alive alongside the volume — easy to get wrong,
/// and impossible to express as a plain owned value without it.
pub struct Fat32 {
    bpb: Bpb,
    /// The volume's length in sectors.
    total_sectors: u32,
    /// The highest valid cluster number, from the volume geometry.
    max_cluster: u32,
    /// Free clusters, from FSINFO. Zero means "not known".
    free_hint: u32,
    /// The medium this volume lives on.
    dev: &'static dyn BlockDevice,
}

impl Fat32 {
    /// Mount a volume from its block device.
    ///
    /// The boot sector is read from sector 0, which is correct for a *partition*
    /// or a superfloppy stick. A whole disk with a partition table needs the
    /// caller to point at the partition, because a physical sector 0 holding an
    /// MBR has no BPB at all.
    pub fn mount(dev: &'static dyn BlockDevice) -> Result<Self, FatError> {
        let mut boot = [0u8; SECTOR_SIZE];
        dev.read_block(0, &mut boot).map_err(FatError::from)?;
        let bpb = Bpb::parse(&boot)?;
        // The 32-bit sector count at offset 32 is the real length on FAT32; the
        // 16-bit one at 19 is zero there. Taking the 16-bit field as the truth is
        // how a 128 MB stick mounts as a 64 MB one and then reports a corrupt
        // cluster chain near the end.
        let total_sectors = u32::from_le_bytes([boot[32], boot[33], boot[34], boot[35]]);
        if total_sectors == 0 {
            return Err(FatError::NotFat32);
        }
        Self::mount_parts(dev, bpb, total_sectors)
    }

    /// Mount from an already-parsed BPB and a known volume length.
    ///
    /// The device must be `'static`, because the returned volume keeps it. Every
    /// caller in this kernel mounts from the driver registry, whose entries are
    /// `'static` statics, so the bound is free in practice.
    pub fn mount_parts(
        dev: &'static dyn BlockDevice,
        bpb: Bpb,
        total_sectors: u32,
    ) -> Result<Self, FatError> {
        let max_cluster = bpb.cluster_count(total_sectors);
        if max_cluster <= FIRST_DATA_CLUSTER {
            return Err(FatError::NotFat32);
        }
        // FSINFO is advisory. A volume with a corrupt one still mounts; it just
        // loses the free-count hint. Demanding it would refuse volumes that are
        // perfectly readable.
        let free_hint = read_fsinfo_free(dev, &bpb).unwrap_or(0);
        Ok(Self { bpb, total_sectors, max_cluster, free_hint, dev })
    }

    /// The volume's BPB.
    #[inline]
    pub const fn bpb(&self) -> &Bpb {
        &self.bpb
    }

    /// The volume length in sectors.
    #[inline]
    pub const fn total_sectors(&self) -> u32 {
        self.total_sectors
    }

    /// The last valid cluster number.
    #[inline]
    pub const fn max_cluster(&self) -> u32 {
        self.max_cluster
    }

    /// The FSINFO free-cluster count, or zero when it is not known.
    #[inline]
    pub const fn free_hint(&self) -> u32 {
        self.free_hint
    }

    /// Is `cluster` a real, allocatable cluster?
    ///
    /// Every cluster number read from disk goes through here. A chain that leaves
    /// the data region reads whatever follows the volume, so this check is what
    /// stops a corrupt chain from becoming a buffer overrun.
    #[inline]
    pub const fn cluster_ok(&self, cluster: u32) -> bool {
        cluster >= FIRST_DATA_CLUSTER && cluster < self.max_cluster
    }

    /// Read one sector of the volume.
    fn read_sector(&self, dev: &dyn BlockDevice, lba: u64, buf: &mut [u8]) -> Result<(), FatError> {
        dev.read_block(lba, buf).map_err(FatError::from)
    }

    /// Write one sector of the volume.
    fn write_sector(&self, dev: &dyn BlockDevice, lba: u64, buf: &[u8]) -> Result<(), FatError> {
        dev.write_block(lba, buf).map_err(FatError::from)
    }

    /// Read one FAT entry.
    ///
    /// Two entries share a sector, so both the sector and the half within it are
    /// computed. Using the entry's own index without halving it reads the wrong
    /// half of the table and follows a plausible-looking chain into unrelated data.
    pub fn get_fat(&self, dev: &dyn BlockDevice, cluster: u32) -> Result<u32, FatError> {
        if cluster > self.max_cluster {
            return Err(FatError::BadCluster);
        }
        let mut buf = [0u8; SECTOR_SIZE];
        self.read_sector(dev, self.bpb.fat_sector_of(cluster), &mut buf)?;
        let off = (cluster as usize % 2) * 4;
        Ok(u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
            & 0x0FFF_FFFF)
    }

    /// Write one FAT entry, mirroring to every FAT.
    ///
    /// Mirroring is not optional here: a volume whose second FAT has drifted is
    /// one `chkdsk` away from losing files, and this is the only place that can
    /// keep the copies equal.
    pub fn set_fat(&self, dev: &dyn BlockDevice, cluster: u32, value: u32) -> Result<(), FatError> {
        if cluster > self.max_cluster {
            return Err(FatError::BadCluster);
        }
        let value = value & 0x0FFF_FFFF;
        for f in 0..self.bpb.num_fats as u64 {
            let base = self.bpb.reserved_sectors as u64 + f * self.bpb.fat_sectors as u64;
            let lba = base + (cluster as u64 / 2) * 4 / SECTOR_SIZE as u64;
            // Read-modify-write: a sector holds two entries and the other must
            // survive.
            let mut buf = [0u8; SECTOR_SIZE];
            self.read_sector(dev, lba, &mut buf)?;
            let off = (cluster as usize % 2) * 4;
            buf[off..off + 4].copy_from_slice(&value.to_le_bytes());
            self.write_sector(dev, lba, &buf)?;
        }
        Ok(())
    }

    /// The cluster after `cluster`, or `None` at the end of the chain.
    ///
    /// A bad-cluster marker ends the chain rather than being followed: continuing
    /// through it lands in arbitrary data, and the file is truncated instead of
    /// the driver reading someone else's bytes.
    pub fn next_cluster(&self, dev: &dyn BlockDevice, cluster: u32) -> Result<Option<u32>, FatError> {
        if !self.cluster_ok(cluster) {
            return Err(FatError::BadCluster);
        }
        let v = self.get_fat(dev, cluster)?;
        Ok(if v >= 0x0FFF_FFF8 { None } else { Some(v) })
    }

    /// Walk a chain, calling `f` with each cluster, up to a hard bound.
    ///
    /// The bound is `max_cluster`, not the file's size: a corrupt chain that loops
    /// back on itself would otherwise spin forever, and a kernel that spins in a
    /// filesystem walk hangs the machine.
    pub fn walk_chain<F>(
        &self,
        dev: &dyn BlockDevice,
        first: u32,
        mut f: F,
    ) -> Result<usize, FatError>
    where
        F: FnMut(u32) -> Result<(), FatError>,
    {
        if !self.cluster_ok(first) {
            return Err(FatError::BadCluster);
        }
        let mut cur = first;
        let mut steps = 0usize;
        while self.cluster_ok(cur) {
            f(cur)?;
            steps += 1;
            if steps > self.max_cluster as usize {
                return Err(FatError::BadCluster);
            }
            match self.next_cluster(dev, cur)? {
                Some(n) => cur = n,
                None => break,
            }
        }
        Ok(steps)
    }
}

/// Read the free-cluster count out of FSINFO, if it is valid.
fn read_fsinfo_free(dev: &dyn BlockDevice, bpb: &Bpb) -> Option<u32> {
    let mut buf = [0u8; SECTOR_SIZE];
    dev.read_block(bpb.fsinfo_sector as u64, &mut buf).ok()?;
    let sig1 = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let sig2 = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    if sig1 != FSINFO_SIG1 || sig2 != FSINFO_SIG2 {
        return None;
    }
    // 0xFFFFFFFF means "unknown", which is what Windows writes after a dirty
    // unmount.
    let free = u32::from_le_bytes([buf[488], buf[489], buf[490], buf[491]]);
    if free == 0xFFFF_FFFF {
        return None;
    }
    Some(free)
}

/// Where a directory entry sits on disk, so it can be rewritten in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryLoc {
    /// LBA of the sector holding the entry.
    pub lba: u64,
    /// The entry's byte offset within that sector.
    pub offset: usize,
}

impl Fat32 {
    /// Read one whole cluster.
    pub fn read_cluster(&self, dev: &dyn BlockDevice, cluster: u32) -> Result<Vec<u8>, FatError> {
        if !self.cluster_ok(cluster) {
            return Err(FatError::BadCluster);
        }
        let n = self.bpb.cluster_bytes() as usize;
        let mut buf = vec![0u8; n];
        let lba = self.bpb.data_sector_of(cluster);
        for s in 0..(n / SECTOR_SIZE) {
            let off = s * SECTOR_SIZE;
            self.read_sector(dev, lba + s as u64, &mut buf[off..off + SECTOR_SIZE])?;
        }
        Ok(buf)
    }

    /// Write one whole cluster.
    pub fn write_cluster(&self, dev: &dyn BlockDevice, cluster: u32, data: &[u8]) -> Result<(), FatError> {
        if !self.cluster_ok(cluster) {
            return Err(FatError::BadCluster);
        }
        let n = self.bpb.cluster_bytes() as usize;
        let mut buf = vec![0u8; n];
        let copy = data.len().min(n);
        buf[..copy].copy_from_slice(&data[..copy]);
        let lba = self.bpb.data_sector_of(cluster);
        for s in 0..(n / SECTOR_SIZE) {
            let off = s * SECTOR_SIZE;
            self.write_sector(dev, lba + s as u64, &buf[off..off + SECTOR_SIZE])?;
        }
        Ok(())
    }

    /// Find one free cluster and link it after `previous`, or start a chain.
    ///
    /// `previous` of zero means "no link to make": the caller is starting a
    /// chain, and the returned cluster has to be marked end-of-chain. Marking it
    /// is the caller's job, not this one's, so the caller decides when the chain
    /// becomes visible in the FAT.
    pub fn alloc_cluster(&self, dev: &dyn BlockDevice, previous: u32) -> Result<u32, FatError> {
        // Start just after the previous cluster to avoid rescanning the free
        // region from the beginning on every allocation: a large file would
        // otherwise be O(n^2) in the number of clusters.
        let start = if previous == 0 { FIRST_DATA_CLUSTER } else { previous + 1 };
        let mut c = start.max(FIRST_DATA_CLUSTER);
        while c < self.max_cluster {
            if self.get_fat(dev, c)? == FREE {
                if previous != 0 {
                    self.set_fat(dev, previous, c)?;
                }
                return Ok(c);
            }
            c += 1;
        }
        // Wrapped past the end: retry from the beginning, in case the free
        // clusters are all below the first one we looked at.
        c = FIRST_DATA_CLUSTER;
        while c < start {
            if self.get_fat(dev, c)? == FREE {
                if previous != 0 {
                    self.set_fat(dev, previous, c)?;
                }
                return Ok(c);
            }
            c += 1;
        }
        Err(FatError::NoSpace)
    }

    /// Allocate `count` clusters, linking them into one chain.
    ///
    /// The last cluster is marked end-of-chain, so the caller can write the first
    /// cluster's number into the directory entry and have a valid file. If
    /// allocation fails part-way the partial chain is returned in the error's
    /// data rather than leaked, because leaking it is what fills a volume with
    /// unreachable clusters that only `chkdsk` can recover.
    pub fn alloc_chain(&self, dev: &dyn BlockDevice, count: u32) -> Result<(u32, u32), FatError> {
        if count == 0 {
            return Ok((0, 0));
        }
        let first = self.alloc_cluster(dev, 0)?;
        let mut prev = first;
        for _ in 1..count {
            match self.alloc_cluster(dev, prev) {
                Ok(c) => prev = c,
                Err(e) => {
                    // Unwind what we took, then report the real failure.
                    self.free_chain_partial(dev, first, prev);
                    return Err(e);
                }
            }
        }
        self.set_fat(dev, prev, EOC)?;
        Ok((first, count))
    }

    /// Release the clusters of a chain, leaving the head marked end-of-chain.
    pub fn free_chain(&self, dev: &dyn BlockDevice, first: u32) -> Result<usize, FatError> {
        if first == 0 || !self.cluster_ok(first) {
            return Ok(0);
        }
        let mut count = 0usize;
        let mut cur = first;
        loop {
            let next = self.next_cluster(dev, cur)?;
            self.set_fat(dev, cur, FREE)?;
            count += 1;
            if count > self.max_cluster as usize {
                break;
            }
            match next {
                Some(n) if self.cluster_ok(n) => cur = n,
                _ => break,
            }
        }
        Ok(count)
    }

    /// Best-effort release of a chain whose tail was never linked.
    fn free_chain_partial(&self, dev: &dyn BlockDevice, first: u32, last: u32) {
        let _ = self.set_fat(dev, last, FREE);
        let _ = self.free_chain(dev, first);
    }

    /// Read every directory entry in `cluster`, with where each sits.
    fn read_dir_raw(
        &self,
        dev: &dyn BlockDevice,
        first: u32,
    ) -> Result<Vec<(DirEntry, EntryLoc)>, FatError> {
        let mut out = Vec::new();
        let per_cluster = self.bpb.cluster_bytes() as usize / 32;
        let base_lba = self.bpb.data_sector_of(first);

        self.walk_chain(dev, first, |cluster| {
            let data = self.read_cluster(dev, cluster)?;
            for i in 0..per_cluster {
                let off = i * 32;
                let e = parse_entry(&data[off..off + 32]);
                let lba = base_lba + (off / SECTOR_SIZE) as u64;
                let within = off % SECTOR_SIZE;
                out.push((e, EntryLoc { lba, offset: within }));
            }
            Ok(())
        })?;

        Ok(out)
    }

    /// The entries of a directory, skipping free, dot and label slots.
    pub fn read_dir(
        &self,
        dev: &dyn BlockDevice,
        first: u32,
    ) -> Result<Vec<(DirEntry, EntryLoc)>, FatError> {
        let all = self.read_dir_raw(dev, first)?;
        Ok(all
            .into_iter()
            .filter(|(e, _)| !e.is_free() && !e.is_dot() && !e.is_volume_label())
            .collect())
    }

    /// Look one name up in a directory.
    pub fn lookup(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
    ) -> Result<(DirEntry, EntryLoc), FatError> {
        for (e, loc) in self.read_dir(dev, dir)? {
            if name_eq(&e, name) {
                return Ok((e, loc));
            }
        }
        Err(FatError::NotFound)
    }

    /// Is a directory free of real entries?
    ///
    /// `rmdir` must not orphan anything, and the only way to know is to look. The
    /// check is over live entries only: a deleted entry does not keep a directory
    /// busy, which is what makes `rm` then `rmdir` work.
    pub fn dir_is_empty(&self, dev: &dyn BlockDevice, dir: u32) -> Result<bool, FatError> {
        Ok(self.read_dir(dev, dir)?.is_empty())
    }

    /// Resolve a `/`-separated path, returning the entry and where it sits.
    ///
    /// Absolute paths start at the root cluster. `.` and `..` are handled: `..` at
    /// the root stays at the root rather than walking off it, which is what a
    /// `cd ..` at the top level must do.
    pub fn resolve(
        &self,
        dev: &dyn BlockDevice,
        path: &str,
    ) -> Result<(DirEntry, EntryLoc), FatError> {
        let mut cur = self.bpb.root_cluster;
        for comp in path.split('/') {
            if comp.is_empty() || comp == "." {
                continue;
            }
            if comp == ".." {
                // The root has no parent, so stay put.
                continue;
            }
            let (e, _loc) = self.lookup(dev, cur, comp)?;
            cur = e.first_cluster();
        }
        // Return a synthetic root entry: the root is a real directory, but it has
        // no entry of its own to point at. The cluster goes through the setter,
        // not the field: FAT32 splits it across two halves.
        let mut root = DirEntry {
            name: *b"           ",
            attr: ATTR_DIRECTORY,
            ..Default::default()
        };
        root.set_first_cluster(cur);
        Ok((root, EntryLoc { lba: 0, offset: 0 }))
    }

    /// Find a free slot in a directory.
    ///
    /// `0x00` marks the end of the directory and `0xE5` a deleted hole. Only a
    /// hole is reused: writing over the end-of-directory marker hides every entry
    /// after it, and on a freshly formatted stick there *is* something after it —
    /// the entry created earlier, which is not at the start.
    fn find_free_slot(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
    ) -> Result<EntryLoc, FatError> {
        let mut hole: Option<EntryLoc> = None;
        let mut at_end: Option<EntryLoc> = None;

        self.walk_chain(dev, dir, |cluster| {
            let lba0 = self.bpb.data_sector_of(cluster);
            for s in 0..(self.bpb.cluster_bytes() as usize / SECTOR_SIZE) {
                let lba = lba0 + s as u64;
                let mut buf = [0u8; SECTOR_SIZE];
                self.read_sector(dev, lba, &mut buf)?;
                for i in 0..DIR_PER_SECTOR {
                    let off = i * 32;
                    let first = buf[off];
                    if first == DELETED_MARK {
                        if hole.is_none() {
                            hole = Some(EntryLoc { lba, offset: off });
                        }
                    } else if first == END_OF_DIR && at_end.is_none() {
                        at_end = Some(EntryLoc { lba, offset: off });
                    }
                }
            }
            Ok(())
        })?;

        // A hole is safer than the end, because it cannot truncate anything.
        hole.or(at_end).ok_or(FatError::NoSpace)
    }

    /// Write a directory entry into a slot, or clear it when `entry` is `None`.
    fn write_slot(
        &self,
        dev: &dyn BlockDevice,
        loc: EntryLoc,
        entry: Option<&DirEntry>,
    ) -> Result<(), FatError> {
        let mut buf = [0u8; SECTOR_SIZE];
        self.read_sector(dev, loc.lba, &mut buf)?;
        if let Some(e) = entry {
            serialize_entry(e, &mut buf[loc.offset..loc.offset + 32]);
        } else {
            // 0xE5, not 0x00: zero would claim the directory ends here.
            buf[loc.offset] = DELETED_MARK;
        }
        self.write_sector(dev, loc.lba, &buf)
    }
}
/// Decode a 32-byte directory entry.
///
/// The offsets come from `struct msdos_dir_entry` in the kernel's `msdos_fs.h`:
/// name at 0..11, attributes at 11, the cluster pair at 20 and 26, size at 28.
pub fn parse_entry(b: &[u8]) -> DirEntry {
    let mut name = [0u8; 11];
    name.copy_from_slice(&b[0..11]);
    DirEntry {
        name,
        attr: b[11],
        size: u32::from_le_bytes([b[28], b[29], b[30], b[31]]),
        cluster_high: u16::from_le_bytes([b[20], b[21]]),
        cluster_low: u16::from_le_bytes([b[26], b[27]]),
    }
}

/// Encode a directory entry into its 32 bytes.
///
/// The buffer is zeroed first because every byte on disk means something: the NT
/// reserved fields, the creation and access timestamps, the write time. Leaving
/// a previous entry's bytes there makes a reader believe a file has creation
/// dates it never had.
pub fn serialize_entry(e: &DirEntry, out: &mut [u8]) {
    out[..32].fill(0);
    out[0..11].copy_from_slice(&e.name);
    out[11] = e.attr;
    out[20..22].copy_from_slice(&e.cluster_high.to_le_bytes());
    out[26..28].copy_from_slice(&e.cluster_low.to_le_bytes());
    out[28..32].copy_from_slice(&e.size.to_le_bytes());
}

impl Fat32 {
    /// Create a file, refusing if the name is taken.
    ///
    /// The chain is allocated first, the data written, and the directory entry
    /// written last. Until that last write the file does not exist, so an I/O
    /// error before it leaves a directory merely missing a file — recoverable —
    /// rather than one listing a file with no data behind it.
    pub fn create_file(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
        data: &[u8],
    ) -> Result<(), FatError> {
        let short = to_83_name(name).ok_or(FatError::BadName)?;
        if self.lookup(dev, dir, name).is_ok() {
            return Err(FatError::Exists);
        }

        let cbytes = self.bpb.cluster_bytes() as usize;
        let count = if data.is_empty() {
            0
        } else {
            ((data.len() + cbytes - 1) / cbytes) as u32
        };
        let (first, _) = self.alloc_chain(dev, count)?;

        let mut written = 0usize;
        let res = self.walk_chain(dev, first, |c| {
            if written >= data.len() {
                return Ok(());
            }
            let end = (written + cbytes).min(data.len());
            self.write_cluster(dev, c, &data[written..end])?;
            written = end;
            Ok(())
        });
        if let Err(e) = res {
            let _ = self.free_chain(dev, first);
            return Err(e);
        }

        let loc = self.find_free_slot(dev, dir)?;
        let mut entry = DirEntry {
            name: short,
            attr: ATTR_ARCHIVE,
            size: data.len() as u32,
            ..Default::default()
        };
        // A zero-length file owns no cluster; writing 2 here would point it at
        // the reserved area.
        if first != 0 {
            entry.set_first_cluster(first);
        }
        self.write_slot(dev, loc, Some(&entry))
    }

    /// Read a file's whole contents.
    pub fn read_file(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
    ) -> Result<Vec<u8>, FatError> {
        let (e, _) = self.lookup(dev, dir, name)?;
        if e.is_dir() {
            return Err(FatError::IsDir);
        }
        let mut out = Vec::with_capacity(e.size as usize);
        self.walk_chain(dev, e.first_cluster(), |c| {
            if out.len() >= e.size as usize {
                return Ok(());
            }
            let data = self.read_cluster(dev, c)?;
            let take = data.len().min(e.size as usize - out.len());
            out.extend_from_slice(&data[..take]);
            Ok(())
        })?;
        Ok(out)
    }

    /// Read a file by full path into `buf`, returning the byte count.
    ///
    /// `buf` is the caller's, and truncates rather than failing when it is too
    /// small — a fixed-size read buffer should not have to grow to read a large
    /// file. The device comes from the mount, so this needs no argument.
    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, FatError> {
        let (e, _) = self.resolve(self.dev, path)?;
        if e.is_dir() {
            return Err(FatError::IsDir);
        }
        let mut written = 0usize;
        let limit = e.size as usize;
        self.walk_chain(self.dev, e.first_cluster(), |c| {
            if written >= buf.len() || written >= limit {
                return Ok(());
            }
            let data = self.read_cluster(self.dev, c)?;
            let take = data.len().min(buf.len() - written).min(limit - written);
            buf[written..written + take].copy_from_slice(&data[..take]);
            written += take;
            Ok(())
        })?;
        Ok(written)
    }

    /// List a directory by full path.
    pub fn list_path(&self, path: &str) -> Result<Vec<(DirEntry, EntryLoc)>, FatError> {
        let (e, _) = self.resolve(self.dev, path)?;
        if !e.is_dir() {
            return Err(FatError::NotDir);
        }
        self.read_dir(self.dev, e.first_cluster())
    }

    /// Create a directory with `.` and `..` entries.
    ///
    /// The two dot entries are not decoration: Windows and most readers require
    /// them, and a directory without `..` is one a recursive delete walks out of.
    pub fn mkdir(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
    ) -> Result<(), FatError> {
        let short = to_83_name(name).ok_or(FatError::BadName)?;
        if self.lookup(dev, dir, name).is_ok() {
            return Err(FatError::Exists);
        }

        let (first, _) = self.alloc_chain(dev, 1)?;
        let loc = match self.find_free_slot(dev, dir) {
            Ok(l) => l,
            Err(e) => {
                let _ = self.free_chain(dev, first);
                return Err(e);
            }
        };

        let mut self_entry = DirEntry { name: *b".          ", attr: ATTR_DIRECTORY, ..Default::default() };
        self_entry.set_first_cluster(first);
        let mut parent_entry = DirEntry { name: *b"..         ", attr: ATTR_DIRECTORY, ..Default::default() };
        parent_entry.set_first_cluster(dir);

        // Write `.` and `..` into the new directory's own first two slots.
        let lba = self.bpb.data_sector_of(first);
        let mut buf = [0u8; SECTOR_SIZE];
        serialize_entry(&self_entry, &mut buf[0..32]);
        serialize_entry(&parent_entry, &mut buf[32..64]);
        self.write_sector(dev, lba, &buf)?;
        for s in 1..(self.bpb.cluster_bytes() as usize / SECTOR_SIZE) {
            self.write_sector(dev, lba + s as u64, &[0u8; SECTOR_SIZE])?;
        }

        let mut entry = DirEntry { name: short, attr: ATTR_DIRECTORY, ..Default::default() };
        entry.set_first_cluster(first);
        if let Err(e) = self.write_slot(dev, loc, Some(&entry)) {
            let _ = self.free_chain(dev, first);
            return Err(e);
        }
        Ok(())
    }

    /// Remove a file, freeing its clusters.
    ///
    /// The entry is marked deleted *first*. If freeing the clusters then fails the
    /// cost is a leak that `chkdsk` recovers; the other order leaves a directory
    /// entry pointing at clusters already handed to a different file, and that
    /// file is now corrupt.
    pub fn unlink(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
    ) -> Result<(), FatError> {
        let (e, loc) = self.lookup(dev, dir, name)?;
        if e.is_dir() {
            return Err(FatError::IsDir);
        }
        self.write_slot(dev, loc, None)?;
        let _ = self.free_chain(dev, e.first_cluster());
        Ok(())
    }

    /// Remove an empty directory.
    ///
    /// Refuses a non-empty directory. Recursing instead would free clusters the
    /// caller has not looked at, and an `rmdir` that silently eats a subtree is
    /// not recoverable from inside the kernel.
    pub fn rmdir(
        &self,
        dev: &dyn BlockDevice,
        dir: u32,
        name: &str,
    ) -> Result<(), FatError> {
        let (e, loc) = self.lookup(dev, dir, name)?;
        if !e.is_dir() {
            return Err(FatError::NotDir);
        }
        if !self.dir_is_empty(dev, e.first_cluster())? {
            return Err(FatError::NotEmpty);
        }
        self.write_slot(dev, loc, None)?;
        let _ = self.free_chain(dev, e.first_cluster());
        Ok(())
    }
}
/// Format `dev` as FAT32.
///
/// A minimal but genuinely valid layout: boot sector, two FATs, FSINFO, a root
/// directory, and a volume label. The point of the 11th onwards is to be readable
/// by Windows — a "filesystem" only this driver can read is not the goal.
pub fn format(
    dev: &dyn BlockDevice,
    total_sectors: u32,
    volume_label: &[u8; 11],
) -> Result<Bpb, FatError> {
    // One sector per cluster, and a FAT with one entry per sector.
    let sectors_per_cluster: u8 = 1;
    let data_sectors = total_sectors as u64;
    let cluster_count = data_sectors / 2; // two FATs
    let fat_sectors = ((cluster_count * 4) / SECTOR_SIZE as u64 + 1) as u32;
    let reserved = 32u16;
    let meta = reserved as u64 + 2 * fat_sectors as u64;

    if meta >= data_sectors {
        return Err(FatError::NoSpace);
    }
    let root_cluster = (meta / sectors_per_cluster as u64 + 2) as u32;

    let mut boot = [0u8; SECTOR_SIZE];
    boot[0..3].copy_from_slice(&[0xEB, 0x58, 0x90]);
    boot[3..11].copy_from_slice(b"TGO-SYS ");
    boot[11..13].copy_from_slice(&(SECTOR_SIZE as u16).to_le_bytes());
    boot[13] = sectors_per_cluster;
    boot[14..16].copy_from_slice(&reserved.to_le_bytes());
    boot[16] = 2; // two FATs
    boot[17..19].copy_from_slice(&0u16.to_le_bytes()); // no fixed root dir on FAT32
    boot[21] = 0xF8; // fixed disk
    boot[24..26].copy_from_slice(&1u16.to_le_bytes()); // sectors per track
    boot[26..28].copy_from_slice(&1u16.to_le_bytes()); // heads
    boot[28..32].copy_from_slice(&0u32.to_le_bytes()); // hidden sectors
    boot[32..36].copy_from_slice(&total_sectors.to_le_bytes());
    boot[36..40].copy_from_slice(&fat_sectors.to_le_bytes());
    boot[44..48].copy_from_slice(&root_cluster.to_le_bytes());
    boot[48..50].copy_from_slice(&1u16.to_le_bytes()); // FSINFO sector
    boot[50..52].copy_from_slice(&6u16.to_le_bytes()); // backup boot sector
    boot[64] = 0x80;
    boot[66] = 0x29; // extended boot signature
    boot[67..71].copy_from_slice(&0x1234_5678u32.to_le_bytes());
    boot[71..82].copy_from_slice(volume_label);
    boot[82..90].copy_from_slice(b"FAT32   ");
    boot[510] = 0x55;
    boot[511] = 0xAA;
    dev.write_block(0, &boot).map_err(FatError::from)?;

    // FSINFO at sector 1.
    let mut fsinfo = [0u8; SECTOR_SIZE];
    fsinfo[0..4].copy_from_slice(&FSINFO_SIG1.to_le_bytes());
    fsinfo[4..8].copy_from_slice(&FSINFO_SIG2.to_le_bytes());
    let free_clusters = (data_sectors - meta - 1).min(u32::MAX as u64) as u32;
    fsinfo[488..492].copy_from_slice(&free_clusters.to_le_bytes());
    fsinfo[510] = 0x55;
    fsinfo[511] = 0xAA;
    dev.write_block(1, &fsinfo).map_err(FatError::from)?;

    dev.write_block(6, &boot).map_err(FatError::from)?;

    // Both FATs zeroed: zero is the "free" marker, so a zeroed FAT is a correct
    // empty one.
    let fat_base = reserved as u64;
    let zero = [0u8; SECTOR_SIZE];
    for f in 0..2u64 {
        for s in 0..fat_sectors as u64 {
            dev.write_block(fat_base + f * fat_sectors as u64 + s, &zero).map_err(FatError::from)?;
        }
    }

    // FAT entries 0 and 1 are reserved, conventionally 0xFFFFF8 and 0xFFFFFFFF.
    // A driver that reads them as data clusters walks into the FAT itself.
    for (idx, val) in [(0u32, 0x0FFF_FFF8u32), (1, 0x0FFF_FFFF)] {
        for f in 0..2u64 {
            let lba = fat_base + f * fat_sectors as u64 + (idx as u64 / 2) * 4 / SECTOR_SIZE as u64;
            let mut buf = [0u8; SECTOR_SIZE];
            dev.read_block(lba, &mut buf).map_err(FatError::from)?;
            let off = (idx as usize % 2) * 4;
            buf[off..off + 4].copy_from_slice(&val.to_le_bytes());
            dev.write_block(lba, &buf).map_err(FatError::from)?;
        }
    }

    let bpb = Bpb {
        bytes_per_sector: SECTOR_SIZE as u16,
        sectors_per_cluster,
        reserved_sectors: reserved,
        num_fats: 2,
        fat_sectors,
        root_cluster,
        fsinfo_sector: 1,
        volume_id: 0x1234_5678,
        volume_label: *volume_label,
        fs_type: *b"FAT32   ",
    };

    // Mark the root end-of-chain and zero its directory.
    let root_lba = bpb.data_sector_of(root_cluster);
    for s in 0..(bpb.cluster_bytes() as usize / SECTOR_SIZE) {
        dev.write_block(root_lba + s as u64, &zero).map_err(FatError::from)?;
    }
    set_fat_raw(dev, &bpb, root_cluster, EOC)?;

    let label = DirEntry { name: pad_83(volume_label), attr: ATTR_VOLUME, ..Default::default() };
    let mut root = [0u8; SECTOR_SIZE];
    serialize_entry(&label, &mut root[0..32]);
    dev.write_block(root_lba, &root).map_err(FatError::from)?;

    Ok(bpb)
}

/// Write one FAT entry using a BPB that has not been mounted yet.
fn set_fat_raw(dev: &dyn BlockDevice, bpb: &Bpb, cluster: u32, val: u32) -> Result<(), FatError> {
    for f in 0..bpb.num_fats as u64 {
        let base = bpb.reserved_sectors as u64 + f * bpb.fat_sectors as u64;
        let lba = base + (cluster as u64 / 2) * 4 / SECTOR_SIZE as u64;
        let mut buf = [0u8; SECTOR_SIZE];
        dev.read_block(lba, &mut buf).map_err(FatError::from)?;
        let off = (cluster as usize % 2) * 4;
        buf[off..off + 4].copy_from_slice(&(val & 0x0FFF_FFFF).to_le_bytes());
        dev.write_block(lba, &buf).map_err(FatError::from)?;
    }
    Ok(())
}

/// Right-pad a label into the 11-byte 8.3 field.
fn pad_83(src: &[u8; 11]) -> [u8; 11] {
    let mut out = [b' '; 11];
    for (i, b) in src.iter().enumerate().take(11) {
        out[i] = b.to_ascii_uppercase();
    }
    out
}
