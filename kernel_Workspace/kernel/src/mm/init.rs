use super::super::abi::abi::*;
use super::super::abi::src::{ring_bytes, RingView};
use super::initabi::*;
use super::initcommand::DsError;
use crate::mm::{phys, space};

const DIRECT_BASE: u64 = 0xFFFF888000000000;

pub struct Driverspace {
    pub aspace: space::AddressSpace,
    pub k2d_phys: u64,
    pub d2k_phys: u64,
    pub params_phys: u64,
    pub scratch_phys: u64,
    pub prepared: bool,
}

static mut DS: Option<Driverspace> = None;

fn kv(phys: u64) -> *mut u8 {
    (DIRECT_BASE + phys) as *mut u8
}

pub fn prepare() -> Result<(), DsError> {
    let aspace = space::AddressSpace::new().ok_or(DsError::NoAspace)?;

    let rb = ring_bytes(DS_RING_CAP);
    let pages = (rb + 4095) / 4096;

    let k2d_phys = phys::alloc_frames(pages).ok_or(DsError::NoMemory)?;
    let d2k_phys = phys::alloc_frames(pages).ok_or(DsError::NoMemory)?;
    let params_phys = phys::alloc_frame().ok_or(DsError::NoMemory)?;

    unsafe {
        core::ptr::write_bytes(kv(k2d_phys), 0, (pages * 4096) as usize);
        core::ptr::write_bytes(kv(d2k_phys), 0, (pages * 4096) as usize);
        core::ptr::write_bytes(kv(params_phys), 0, 4096);

        RingView::new(kv(k2d_phys)).init(DS_RING_CAP);
        RingView::new(kv(d2k_phys)).init(DS_RING_CAP);

        let p = kv(params_phys) as *mut DsInitParams;

        (*p).magic = DS_MAGIC;
        (*p).version = DS_VERSION;
        (*p).k2d_va = DS_K2D_VA;
        (*p).d2k_va = DS_D2K_VA;
        (*p).ring_cap = DS_RING_CAP;
        (*p).ds_va_base = 0x4000_0000;
        (*p).ds_va_size = 0x1000_0000;
    }

    let prot = space::PROT_READ | space::PROT_WRITE | space::PROT_USER;

    if !aspace.map_phys(DS_K2D_VA, k2d_phys, pages * 4096, prot) {
        return Err(DsError::NoAspace);
    }

    if !aspace.map_phys(DS_D2K_VA, d2k_phys, pages * 4096, prot) {
        return Err(DsError::NoAspace);
    }

    if !aspace.map_phys(DS_INIT_PARAMS_VA, params_phys, 4096, prot) {
        return Err(DsError::NoAspace);
    }

    let scratch_phys = phys::alloc_frame().ok_or(DsError::NoMemory)?;

    unsafe {
        core::ptr::write_bytes(kv(scratch_phys), 0, 4096);
    }

    if !aspace.map_phys(DS_SCRATCH_VA, scratch_phys, 4096, prot) {
        return Err(DsError::NoAspace);
    }

    unsafe {
        DS = Some(Driverspace {
            aspace,
            k2d_phys,
            d2k_phys,
            params_phys,
            scratch_phys,
            prepared: true,
        });
    }

    Ok(())
}

pub fn self_test() -> Result<(), DsError> {
    let ds = unsafe { DS.as_ref().ok_or(DsError::NotPrepared)? };

    let k2d = unsafe { RingView::new(kv(ds.k2d_phys)) };
    let d2k = unsafe { RingView::new(kv(ds.d2k_phys)) };

    let msg = DsMsg {
        id: 1,
        cmd: DsCmd::Init as u32,
        flags: DS_FLAG_RESPONSE,
        arg0: DS_INIT_PARAMS_VA,
        arg1: DS_MAGIC,
        arg2: DS_VERSION as u64,
        status: 0,
        pad: 0,
    };

    if !k2d.push(&msg) {
        return Err(DsError::QueueFull);
    }

    let got = k2d.pop().ok_or(DsError::Timeout)?;

    if got.cmd != DsCmd::Init as u32 || got.arg1 != DS_MAGIC {
        return Err(DsError::BadStatus(-1));
    }

    let mut resp = got;
    resp.status = 0;

    if !d2k.push(&resp) {
        return Err(DsError::QueueFull);
    }

    let back = d2k.pop().ok_or(DsError::Timeout)?;

    if back.id != 1 {
        return Err(DsError::BadStatus(-2));
    }

    Ok(())
}

pub fn ready() -> bool {
    unsafe { DS.as_ref().map(|d| d.prepared).unwrap_or(false) }
}

pub fn k2d_view() -> Option<RingView> {
    unsafe { DS.as_ref().map(|d| RingView::new(kv(d.k2d_phys))) }
}

pub fn d2k_view() -> Option<RingView> {
    unsafe { DS.as_ref().map(|d| RingView::new(kv(d.d2k_phys))) }
}

pub fn scratch_view() -> Option<*mut u8> {
    unsafe { DS.as_ref().map(|d| kv(d.scratch_phys)) }
}

pub fn map_into_ds(va: u64, phys: u64, len: usize, prot: space::ProtFlags) -> bool {
    unsafe {
        match DS.as_ref() {
            Some(d) => d.aspace.map_phys(va, phys, len, prot),
            None => false,
        }
    }
}

pub fn unmap_from_ds(va: u64, len: usize) -> bool {
    unsafe {
        match DS.as_ref() {
            Some(d) => d.aspace.munmap(va, len),
            None => false,
        }
    }
}