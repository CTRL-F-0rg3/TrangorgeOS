//! Ergonomic logging macros.

#[macro_export]
macro_rules! ds_log {
    ($level:expr, $module:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let mut buf = $crate::formatter::LogBuffer::new();
        let _ = write!(buf, $($arg)*);
        $crate::transport::emit_log($level, $module, &buf);
    }};
}

#[macro_export]
macro_rules! ds_trace { ($($arg:tt)*) => ($crate::ds_log!($crate::levels::LogLevel::Trace, module_path!(), $($arg)*)); }

#[macro_export]
macro_rules! ds_info {  ($($arg:tt)*) => ($crate::ds_log!($crate::levels::LogLevel::Info,  module_path!(), $($arg)*)); }

#[macro_export]
macro_rules! ds_warn {  ($($arg:tt)*) => ($crate::ds_log!($crate::levels::LogLevel::Warn,  module_path!(), $($arg)*)); }

#[macro_export]
macro_rules! ds_error { ($($arg:tt)*) => ($crate::ds_log!($crate::levels::LogLevel::Error, module_path!(), $($arg)*)); }

#[macro_export]
macro_rules! ds_fatal { ($($arg:tt)*) => ($crate::ds_log!($crate::levels::LogLevel::Fatal, module_path!(), $($arg)*)); }
