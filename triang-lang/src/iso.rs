pub struct IsoFile {
    pub name: String,
    pub data: Vec<u8>,
}

const SECTOR: usize = 2048;

fn dstring(s: &str, len: usize) -> Vec<u8> {
    let mut v = vec![b' '; len];
    for (i, b) in s.bytes().enumerate() {
        if i < len {
            v[i] = b;
        }
    }
    v
}

fn push_both_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
    out.extend_from_slice(&v.to_be_bytes());
}

fn push_both_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
    out.extend_from_slice(&v.to_be_bytes());
}

fn put_both_u16(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
    buf[off + 2..off + 4].copy_from_slice(&v.to_be_bytes());
}

fn put_both_u32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    buf[off + 4..off + 8].copy_from_slice(&v.to_be_bytes());
}

fn dir_record(name: &[u8], extent: u32, size: u32, flags: u8) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(0);
    r.push(0);
    push_both_u32(&mut r, extent);
    push_both_u32(&mut r, size);
    r.extend_from_slice(&[0u8; 7]);
    r.push(flags);
    r.push(0);
    r.push(0);
    push_both_u16(&mut r, 1);
    r.push(name.len() as u8);
    r.extend_from_slice(name);
    r[0] = (33 + name.len()) as u8;
    if (33 + name.len()) % 2 == 1 {
        r.push(0);
    }
    r
}

pub fn build(volume: &str, files: &[IsoFile]) -> Vec<u8> {
    let root_sector: u32 = 20;
    let mut next_sector: u32 = 21;

    let mut extents: Vec<(u32, u32)> = Vec::new();
    for f in files {
        extents.push((next_sector, f.data.len() as u32));
        next_sector += ((f.data.len() + SECTOR - 1) / SECTOR) as u32;
    }

    let total = next_sector;

    let mut iso: Vec<u8> = Vec::new();
    iso.extend_from_slice(&vec![0u8; 16 * SECTOR]);

    let mut pvd = vec![0u8; SECTOR];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 1;
    pvd[8..40].copy_from_slice(&dstring(volume, 32));
    put_both_u32(&mut pvd, 40, total);
    put_both_u16(&mut pvd, 80, 1);
    put_both_u16(&mut pvd, 84, 1);
    put_both_u16(&mut pvd, 88, SECTOR as u16);
    put_both_u32(&mut pvd, 92, 10);
    pvd[100..104].copy_from_slice(&18u32.to_le_bytes());
    pvd[104..108].copy_from_slice(&19u32.to_be_bytes());
    let root_rec = dir_record(&[0u8], root_sector, SECTOR as u32, 2);
    pvd[108..108 + root_rec.len()].copy_from_slice(&root_rec);
    pvd[833] = 1;
    iso.extend_from_slice(&pvd);

    let mut term = vec![0u8; SECTOR];
    term[0] = 255;
    term[1..6].copy_from_slice(b"CD001");
    term[6] = 1;
    iso.extend_from_slice(&term);

    let mut lpath = vec![0u8; SECTOR];
    lpath[0] = 1;
    lpath[2..6].copy_from_slice(&root_sector.to_le_bytes());
    lpath[6..8].copy_from_slice(&1u16.to_le_bytes());
    iso.extend_from_slice(&lpath);

    let mut mpath = vec![0u8; SECTOR];
    mpath[0] = 1;
    mpath[2..6].copy_from_slice(&root_sector.to_be_bytes());
    mpath[6..8].copy_from_slice(&1u16.to_be_bytes());
    iso.extend_from_slice(&mpath);

    let mut root = Vec::new();
    root.extend_from_slice(&dir_record(&[0u8], root_sector, SECTOR as u32, 2));
    root.extend_from_slice(&dir_record(&[1u8], root_sector, SECTOR as u32, 2));
    for (i, f) in files.iter().enumerate() {
        root.extend_from_slice(&dir_record(f.name.as_bytes(), extents[i].0, extents[i].1, 0));
    }
    while root.len() < SECTOR {
        root.push(0);
    }
    iso.extend_from_slice(&root);

    for f in files {
        iso.extend_from_slice(&f.data);
        while iso.len() % SECTOR != 0 {
            iso.push(0);
        }
    }

    iso
}