use crate::traits::Write;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
    fn serial_write_str(s: *const u8);
}

pub struct Serial;
impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        let bytes = s.as_bytes();
        if bytes.is_empty() { return Ok(()); }
        unsafe { 
            let mut null_terminated = [0u8; 4096];
            let len = bytes.len().min(4095);
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), null_terminated.as_mut_ptr(), len);
            null_terminated[len] = 0;
            serial_write_str(null_terminated.as_ptr());
        }
        Ok(())
    }
}

pub struct VgaConsole;
impl Write for VgaConsole {
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        let fmt = b"%s\0".as_ptr();
        let mut null_terminated = [0u8; 4096];
        let len = s.as_bytes().len().min(4095);
        unsafe {
            core::ptr::copy_nonoverlapping(s.as_ptr(), null_terminated.as_mut_ptr(), len);
            null_terminated[len] = 0;
            kprintf(fmt, null_terminated.as_ptr());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use $crate::traits::Write;
            let mut console = $crate::console::Serial;
            let _ = core::write!(console, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {
        {
            use $crate::traits::Write;
            let mut console = $crate::console::Serial;
            let _ = core::write!(console, $($arg)*);
            let _ = console.write_str("\n");
        }
    };
}