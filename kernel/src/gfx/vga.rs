use x86_64::instructions::port::Port;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    Mode13h,
    Mode12h,
}

fn write_regs(misc: u8, seq: &[u8], crtc: &[u8], gfx: &[u8], attr: &[u8]) {
    unsafe {
        Port::<u8>::new(0x3C2).write(misc);

        for (i, v) in seq.iter().enumerate() {
            Port::<u8>::new(0x3C4).write(i as u8);
            Port::<u8>::new(0x3C5).write(*v);
        }

        Port::<u8>::new(0x3D4).write(0x03);
        let v = Port::<u8>::new(0x3D5).read();
        Port::<u8>::new(0x3D5).write(v | 0x80);
        Port::<u8>::new(0x3D4).write(0x11);
        let v = Port::<u8>::new(0x3D5).read();
        Port::<u8>::new(0x3D5).write(v & !0x80);

        for (i, v) in crtc.iter().enumerate() {
            Port::<u8>::new(0x3D4).write(i as u8);
            Port::<u8>::new(0x3D5).write(*v);
        }

        for (i, v) in gfx.iter().enumerate() {
            Port::<u8>::new(0x3CE).write(i as u8);
            Port::<u8>::new(0x3CF).write(*v);
        }


        for (i, v) in attr.iter().enumerate() {
            let _ = Port::<u8>::new(0x3DA).read();
            Port::<u8>::new(0x3C0).write(i as u8);
            Port::<u8>::new(0x3C0).write(*v);
        }

        let _ = Port::<u8>::new(0x3DA).read();
        Port::<u8>::new(0x3C0).write(0x20);
    }
}

pub fn set_mode(mode: VideoMode) {
    match mode {
        VideoMode::Mode13h => write_regs(
            0x63,
            &[0x03, 0x01, 0x0F, 0x00, 0x0E],
            &[0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F, 0x00, 0x41,
              0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9C, 0x0E, 0x8F, 0x28,
              0x40, 0x96, 0xB9, 0xA3, 0xFF],
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F, 0xFF],
            &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
              0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x41, 0x00, 0x0F, 0x00, 0x00],
        ),
        VideoMode::Mode12h => write_regs(
            0xE3,
            &[0x03, 0x01, 0x0F, 0x00, 0x06],
            &[0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0x0B, 0x3E, 0x00, 0x40,
              0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xEA, 0x0C, 0xDF, 0x28,
              0x00, 0xE7, 0x04, 0xE3, 0xFF],
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x0F, 0xFF],
            &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
              0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x01, 0x00, 0x0F, 0x00, 0x00],
        ),
    }
}


const DISPI_INDEX_PORT: u16 = 0x01CE;
const DISPI_DATA_PORT: u16 = 0x01CF;

const DISPI_INDEX_ID: u16 = 0;
const DISPI_INDEX_XRES: u16 = 1;
const DISPI_INDEX_YRES: u16 = 2;
const DISPI_INDEX_BPP: u16 = 3;
const DISPI_INDEX_ENABLE: u16 = 4;
const DISPI_INDEX_VIRT_WIDTH: u16 = 6;
const DISPI_INDEX_VIRT_HEIGHT: u16 = 7;
const DISPI_INDEX_X_OFFSET: u16 = 8;
const DISPI_INDEX_Y_OFFSET: u16 = 9;

const DISPI_DISABLED: u16 = 0x00;
const DISPI_ENABLED: u16 = 0x01;
const DISPI_LFB_ENABLED: u16 = 0x40;
const DISPI_NOCLEARMEM: u16 = 0x80;

fn dispi_write(index: u16, value: u16) {
    unsafe {
        Port::<u16>::new(DISPI_INDEX_PORT).write(index);
        Port::<u16>::new(DISPI_DATA_PORT).write(value);
    }
}

fn dispi_read(index: u16) -> u16 {
    unsafe {
        Port::<u16>::new(DISPI_INDEX_PORT).write(index);
        Port::<u16>::new(DISPI_DATA_PORT).read()
    }
}

pub fn bochs_version() -> Option<u16> {
    let id = dispi_read(DISPI_INDEX_ID);
    if (0xB0C0..=0xB0C5).contains(&id) {
        Some(id)
    } else {
        None
    }
}

pub fn bochs_disable() {
    if bochs_version().is_some() {
        dispi_write(DISPI_INDEX_ENABLE, DISPI_DISABLED);
    }
}

pub fn bochs_set_mode(width: u32, height: u32, bpp: u32) -> bool {
    if bochs_version().is_none() {
        return false;
    }

    dispi_write(DISPI_INDEX_ENABLE, DISPI_DISABLED);
    dispi_write(DISPI_INDEX_XRES, width as u16);
    dispi_write(DISPI_INDEX_YRES, height as u16);
    dispi_write(DISPI_INDEX_BPP, bpp as u16);
    dispi_write(DISPI_INDEX_VIRT_WIDTH, 0);
    dispi_write(DISPI_INDEX_VIRT_HEIGHT, 0);
    dispi_write(DISPI_INDEX_X_OFFSET, 0);
    dispi_write(DISPI_INDEX_Y_OFFSET, 0);
    dispi_write(DISPI_INDEX_ENABLE, DISPI_ENABLED | DISPI_LFB_ENABLED | DISPI_NOCLEARMEM);


    dispi_read(DISPI_INDEX_XRES) == width as u16
}

pub fn bochs_lfb_base() -> Option<u64> {
    for dev in crate::pci::enumerate() {
        if dev.vendor_id == 0x1234 && dev.device_id == 0x1111 && dev.class_code == 0x03 {
            let bar0 = crate::pci::bar(dev.address, 0);
            if let Some(base) = crate::pci::mem_base_from_bar(bar0) {
                return Some(base as u64);
            }
        }
    }
    None
}
