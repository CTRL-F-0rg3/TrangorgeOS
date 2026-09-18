pub unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port);
    v
}

pub unsafe fn outb(port: u16, v: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") v);
}

pub unsafe fn inw(port: u16) -> u16 {
    let v: u16;
    core::arch::asm!("in ax, dx", out("ax") v, in("dx") port);
    v
}

pub unsafe fn outw(port: u16, v: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") v);
}
