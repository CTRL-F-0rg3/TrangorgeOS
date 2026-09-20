// driverspace_workspace/lib/ds-log/src/macros.rs

#[macro_export]
macro_rules! ds_log {
    ($level:expr, $($arg:tt)*) => {{
        let mut buf = $crate::formatter::LogBuffer::new();
        let _ = core::fmt::write(&mut buf, format_args!($($arg)*));
        $crate::transport::emit_log($level, "", &buf);
    }};
}

#[macro_export]
macro_rules! ds_trace {
    ($($arg:tt)*) => { $crate::ds_log!($crate::LogLevel::Trace, $($arg)*) };
}

#[macro_export]
macro_rules! ds_info {
    ($($arg:tt)*) => { $crate::ds_log!($crate::LogLevel::Info, $($arg)*) };
}

#[macro_export]
macro_rules! ds_warn {
    ($($arg:tt)*) => { $crate::ds_log!($crate::LogLevel::Warn, $($arg)*) };
}

#[macro_export]
macro_rules! ds_error {
    ($($arg:tt)*) => { $crate::ds_log!($crate::LogLevel::Error, $($arg)*) };
}

#[macro_export]
macro_rules! ds_fatal {
    ($($arg:tt)*) => { $crate::ds_log!($crate::LogLevel::Fatal, $($arg)*) };
}