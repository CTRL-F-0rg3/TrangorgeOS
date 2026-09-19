#[macro_export]
macro_rules! kassert {
    ($cond:expr) => {
        if !$cond {
            panic!("kassert failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("kassert failed: {} - {}", stringify!($cond), $msg);
        }
    };
}

#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            kstd_io::kprintln!("[DBG] {}", core::format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        kstd_io::kprintln!("[INF] {}", core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        kstd_io::kprintln!("[WRN] {}", core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        kstd_io::kprintln!("[ERR] {}", core::format_args!($($arg)*));
    };
}