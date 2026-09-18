//! Log severity levels.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Info  = 1,
    Warn  = 2,
    Error = 3,
    Fatal = 4,
}

impl LogLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Info  => "INFO ",
            Self::Warn  => "WARN ",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}