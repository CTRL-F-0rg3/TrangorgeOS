//! win_kernel_bridge.rs
//!
//! Teoretyczna warstwa komunikacyjna Rust -> warstwa kompatybilności C -> kernel.
//!
//! Projekt jest celowo podzielony na trzy poziomy:
//! 1. `KernelTransport` — czysty kontrakt transportu.
//! 2. `CompatibilityBridge` — bezpieczne API Rust do request/response.
//! 3. `extern "C"` — stabilny punkt eksportu dla C, C++ albo istniejącego runtime'u.
//!
//! Prawdziwy backend Windows należy podpiąć w `WindowsKernelTransport`.
//! Najczęściej będzie to wrapper nad `CreateFileW` + `DeviceIoControl`,
//! ale kod nie zakłada konkretnego modelu sterownika ani konkretnego IOCTL.

#![allow(dead_code)]

use std::fmt;
use std::ptr;
use std::slice;
use std::sync::{Arc, Mutex};

/* ========================================================================== */
/*                               ABI i stałe                                  */
/* ========================================================================== */

/// Wersja protokołu między warstwą Rust a backendem kernela.
pub const BRIDGE_ABI_MAJOR: u16 = 1;
pub const BRIDGE_ABI_MINOR: u16 = 0;
pub const BRIDGE_MAX_MESSAGE: usize = 1024 * 1024;
pub const BRIDGE_DEVICE_PATH_MAX: usize = 260;

/// Kody błędów eksportowane przez C ABI. Wartości są stabilne i nie powinny
/// być zmieniane po rozpoczęciu używania biblioteki przez zewnętrzny kod.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeStatus {
    Ok = 0,
    InvalidArgument = -22,
    OutOfMemory = -12,
    NotImplemented = -38,
    NotConnected = -107,
    IoError = -5,
    Timeout = -110,
    BufferTooSmall = -75,
    ProtocolError = -71,
    Busy = -16,
    Internal = -255,
}

impl BridgeStatus {
    pub const fn as_i32(self) -> i32 { self as i32 }
}

impl fmt::Display for BridgeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Ok => "success",
            Self::InvalidArgument => "invalid argument",
            Self::OutOfMemory => "out of memory",
            Self::NotImplemented => "not implemented",
            Self::NotConnected => "not connected",
            Self::IoError => "I/O error",
            Self::Timeout => "timeout",
            Self::BufferTooSmall => "buffer too small",
            Self::ProtocolError => "protocol error",
            Self::Busy => "busy",
            Self::Internal => "internal error",
        };
        f.write_str(text)
    }
}

/// Rodzaje operacji, które może obsługiwać sterownik/backend.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Handshake = 0x0001,
    QueryVersion = 0x0002,
    Read = 0x0010,
    Write = 0x0011,
    Ioctl = 0x0012,
    MapMemory = 0x0020,
    UnmapMemory = 0x0021,
    Event = 0x0030,
    Shutdown = 0x00ff,
}

/// Flagi komunikatu.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MessageFlags(pub u32);

impl MessageFlags {
    pub const NONE: Self = Self(0);
    pub const REQUEST: Self = Self(1 << 0);
    pub const RESPONSE: Self = Self(1 << 1);
    pub const ASYNC: Self = Self(1 << 2);
    pub const NEEDS_REPLY: Self = Self(1 << 3);
    pub const USER_BUFFER: Self = Self(1 << 4);
    pub const KERNEL_BUFFER: Self = Self(1 << 5);

    pub const fn contains(self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
}

/// Nagłówek jest jawnie `repr(C)`, aby mógł być współdzielony z C/kernel ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MessageHeader {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub operation: u32,
    pub flags: u32,
    pub sequence: u64,
    pub payload_len: u32,
    pub status: i32,
}

impl MessageHeader {
    pub fn request(operation: Operation, sequence: u64, payload_len: usize) -> Result<Self, BridgeError> {
        let payload_len = u32::try_from(payload_len).map_err(|_| BridgeError::MessageTooLarge)?;
        Ok(Self {
            abi_major: BRIDGE_ABI_MAJOR,
            abi_minor: BRIDGE_ABI_MINOR,
            operation: operation as u32,
            flags: MessageFlags::REQUEST.union(MessageFlags::NEEDS_REPLY).0,
            sequence,
            payload_len,
            status: BridgeStatus::Ok.as_i32(),
        })
    }

    pub fn validate(&self, actual_payload_len: usize) -> Result<(), BridgeError> {
        if self.abi_major != BRIDGE_ABI_MAJOR { return Err(BridgeError::AbiMismatch); }
        if self.payload_len as usize != actual_payload_len { return Err(BridgeError::Protocol); }
        if actual_payload_len > BRIDGE_MAX_MESSAGE { return Err(BridgeError::MessageTooLarge); }
        Ok(())
    }
}

/// Bufor komunikatu. Nagłówek jest osobno, dzięki czemu transport może
/// używać własnego framingu, IOCTL albo kolejek ring-buffer.
#[derive(Clone, Debug)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(operation: Operation, sequence: u64, payload: &[u8]) -> Result<Self, BridgeError> {
        if payload.len() > BRIDGE_MAX_MESSAGE { return Err(BridgeError::MessageTooLarge); }
        Ok(Self { header: MessageHeader::request(operation, sequence, payload.len())?, payload: payload.to_vec() })
    }
}

/* ========================================================================== */
/*                               Błędy Rust                                   */
/* ========================================================================== */

#[derive(Debug)]
pub enum BridgeError {
    InvalidArgument(&'static str),
    MessageTooLarge,
    BufferTooSmall { required: usize, provided: usize },
    AbiMismatch,
    Protocol,
    NotConnected,
    NotImplemented,
    Io(i32),
    Synchronization,
    Internal(&'static str),
}

impl BridgeError {
    pub const fn status(&self) -> BridgeStatus {
        match self {
            Self::InvalidArgument(_) => BridgeStatus::InvalidArgument,
            Self::MessageTooLarge => BridgeStatus::BufferTooSmall,
            Self::BufferTooSmall { .. } => BridgeStatus::BufferTooSmall,
            Self::AbiMismatch | Self::Protocol => BridgeStatus::ProtocolError,
            Self::NotConnected => BridgeStatus::NotConnected,
            Self::NotImplemented => BridgeStatus::NotImplemented,
            Self::Io(_) => BridgeStatus::IoError,
            Self::Synchronization | Self::Internal(_) => BridgeStatus::Internal,
        }
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(v) => write!(f, "invalid argument: {v}"),
            Self::BufferTooSmall { required, provided } => write!(f, "buffer too small: required={required}, provided={provided}"),
            Self::Io(code) => write!(f, "transport I/O error: {code}"),
            Self::Internal(v) => write!(f, "internal error: {v}"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for BridgeError {}

/* ========================================================================== */
/*                         Kontrakt transportu                                */
/* ========================================================================== */

/// Backend transportowy nie zna logiki wyższej warstwy. Ma jedynie wysłać
/// jeden komunikat i zwrócić odpowiedź.
pub trait KernelTransport: Send + Sync {
    fn connect(&mut self) -> Result<(), BridgeError>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn transact(&mut self, request: &Message) -> Result<Message, BridgeError>;
}

/// Adapter do wcześniejszego C `compat.h`. Deklaracje są celowo minimalne;
/// uchwyt może wskazywać na własny runtime, a niekoniecznie na prawdziwy Linux.
#[repr(C)]
pub struct CompatKernel {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CompatKernelConfig {
    pub page_size: usize,
    pub cpu_count: u32,
    pub max_tasks: u32,
    pub max_handles: u32,
    pub monotonic_hz: u64,
    pub deterministic_time: bool,
}

extern "C" {
    fn lc_kernel_init(
        out: *mut *mut CompatKernel,
        config: *const CompatKernelConfig,
        hooks: *const core::ffi::c_void,
    ) -> i32;
    fn lc_kernel_shutdown(kernel: *mut CompatKernel);
}

/// Opcjonalny uchwyt do poprzedniej warstwy C. Nie wykonuje automatycznego
/// mapowania do Windows; tylko zachowuje granicę ABI.
pub struct CompatBackend {
    kernel: *mut CompatKernel,
    connected: bool,
}

unsafe impl Send for CompatBackend {}
unsafe impl Sync for CompatBackend {}

impl CompatBackend {
    pub fn from_raw(kernel: *mut CompatKernel) -> Result<Self, BridgeError> {
        if kernel.is_null() { return Err(BridgeError::InvalidArgument("null compat kernel")); }
        Ok(Self { kernel, connected: true })
    }

    pub fn raw(&self) -> *mut CompatKernel { self.kernel }
}

impl Drop for CompatBackend {
    fn drop(&mut self) {
        if !self.kernel.is_null() {
            // Własność uchwytu jest jawna: backend zwalnia go przy drop.
            unsafe { lc_kernel_shutdown(self.kernel); }
            self.kernel = ptr::null_mut();
        }
    }
}

impl KernelTransport for CompatBackend {
    fn connect(&mut self) -> Result<(), BridgeError> { self.connected = true; Ok(()) }
    fn disconnect(&mut self) { self.connected = false; }
    fn is_connected(&self) -> bool { self.connected && !self.kernel.is_null() }
    fn transact(&mut self, _request: &Message) -> Result<Message, BridgeError> {
        // Tu należy podpiąć własny eksport z compat.c, np. lc_kernel_ioctl.
        Err(BridgeError::NotImplemented)
    }
}

/* ========================================================================== */
/*                      Backend Windows-kernel                                */
/* ========================================================================== */

/// Konfiguracja urządzenia/sterownika. `device_path` powinien być nazwą
/// symboliczną urządzenia, np. przekazaną przez użytkownika runtime'u.
#[derive(Clone, Debug)]
pub struct WindowsKernelConfig {
    pub device_path: String,
    pub ioctl_base: u32,
    pub timeout_ms: u32,
}

impl Default for WindowsKernelConfig {
    fn default() -> Self {
        Self { device_path: String::from("\\\\.\\TheoreticalCompat"), ioctl_base: 0x800, timeout_ms: 5_000 }
    }
}

pub struct WindowsKernelTransport {
    config: WindowsKernelConfig,
    connected: bool,
    #[cfg(windows)]
    handle: *mut core::ffi::c_void,
}

impl WindowsKernelTransport {
    pub fn new(config: WindowsKernelConfig) -> Self {
        Self {
            config,
            connected: false,
            #[cfg(windows)]
            handle: ptr::null_mut(),
        }
    }

    pub fn config(&self) -> &WindowsKernelConfig { &self.config }

    #[cfg(not(windows))]
    fn platform_connect(&mut self) -> Result<(), BridgeError> { Err(BridgeError::NotImplemented) }

    #[cfg(windows)]
    fn platform_connect(&mut self) -> Result<(), BridgeError> {
        // TODO: podpiąć CreateFileW i zachować uchwyt urządzenia.
        // Nie wpisujemy tu arbitralnych wywołań do kernela bez znajomości
        // kontraktu sterownika oraz bezpieczeństwa buforów.
        let _ = &self.config;
        Err(BridgeError::NotImplemented)
    }

    #[cfg(not(windows))]
    fn platform_transact(&mut self, _request: &Message) -> Result<Message, BridgeError> { Err(BridgeError::NotImplemented) }

    #[cfg(windows)]
    fn platform_transact(&mut self, _request: &Message) -> Result<Message, BridgeError> {
        // TODO: DeviceIoControl(handle, ioctl, input, output, timeout).
        Err(BridgeError::NotImplemented)
    }
}

impl KernelTransport for WindowsKernelTransport {
    fn connect(&mut self) -> Result<(), BridgeError> {
        self.platform_connect()?;
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) { self.connected = false; }
    fn is_connected(&self) -> bool { self.connected }

    fn transact(&mut self, request: &Message) -> Result<Message, BridgeError> {
        if !self.connected { return Err(BridgeError::NotConnected); }
        request.header.validate(request.payload.len())?;
        self.platform_transact(request)
    }
}

/* ========================================================================== */
/*                         Bezpieczny bridge Rust                             */
/* ========================================================================== */

pub struct CompatibilityBridge<T: KernelTransport> {
    transport: Arc<Mutex<T>>,
    next_sequence: u64,
}

impl<T: KernelTransport> CompatibilityBridge<T> {
    pub fn new(transport: T) -> Self { Self { transport: Arc::new(Mutex::new(transport)), next_sequence: 1 } }

    pub fn connect(&mut self) -> Result<(), BridgeError> {
        self.transport.lock().map_err(|_| BridgeError::Synchronization)?.connect()
    }

    pub fn disconnect(&mut self) {
        if let Ok(mut transport) = self.transport.lock() { transport.disconnect(); }
    }

    pub fn is_connected(&self) -> bool {
        self.transport.lock().map(|transport| transport.is_connected()).unwrap_or(false)
    }

    pub fn transact(&mut self, operation: Operation, payload: &[u8]) -> Result<Vec<u8>, BridgeError> {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1).max(1);
        let request = Message::new(operation, sequence, payload)?;
        let mut transport = self.transport.lock().map_err(|_| BridgeError::Synchronization)?;
        let response = transport.transact(&request)?;
        response.header.validate(response.payload.len())?;
        if response.header.sequence != sequence { return Err(BridgeError::Protocol); }
        if response.header.status != BridgeStatus::Ok.as_i32() { return Err(BridgeError::Io(response.header.status)); }
        Ok(response.payload)
    }

    pub fn query_version(&mut self) -> Result<Vec<u8>, BridgeError> { self.transact(Operation::QueryVersion, &[]) }
    pub fn read(&mut self, request: &[u8]) -> Result<Vec<u8>, BridgeError> { self.transact(Operation::Read, request) }
    pub fn write(&mut self, request: &[u8]) -> Result<Vec<u8>, BridgeError> { self.transact(Operation::Write, request) }
    pub fn ioctl(&mut self, request: &[u8]) -> Result<Vec<u8>, BridgeError> { self.transact(Operation::Ioctl, request) }
}

/* ========================================================================== */
/*                     Minimalny eksport C ABI                                */
/* ========================================================================== */

/// Opaque bridge widziany z C. Nie eksportujemy layoutu trait objectu.
#[repr(C)]
pub struct RustBridgeHandle {
    bridge: Mutex<CompatibilityBridge<WindowsKernelTransport>>,
}

#[no_mangle]
pub extern "C" fn rust_bridge_create(out: *mut *mut RustBridgeHandle) -> i32 {
    if out.is_null() { return BridgeStatus::InvalidArgument.as_i32(); }
    let bridge = CompatibilityBridge::new(WindowsKernelTransport::new(WindowsKernelConfig::default()));
    let boxed = Box::new(RustBridgeHandle { bridge: Mutex::new(bridge) });
    unsafe { *out = Box::into_raw(boxed); }
    BridgeStatus::Ok.as_i32()
}

#[no_mangle]
pub extern "C" fn rust_bridge_destroy(handle: *mut RustBridgeHandle) {
    if !handle.is_null() { unsafe { drop(Box::from_raw(handle)); } }
}

#[no_mangle]
pub extern "C" fn rust_bridge_connect(handle: *mut RustBridgeHandle) -> i32 {
    if handle.is_null() { return BridgeStatus::InvalidArgument.as_i32(); }
    let h = unsafe { &*handle };
    match h.bridge.lock().map_err(|_| BridgeError::Synchronization).and_then(|mut b| b.connect()) {
        Ok(()) => BridgeStatus::Ok.as_i32(), Err(e) => e.status().as_i32(),
    }
}

#[no_mangle]
pub extern "C" fn rust_bridge_disconnect(handle: *mut RustBridgeHandle) {
    if handle.is_null() { return; }
    let h = unsafe { &*handle };
    if let Ok(mut bridge) = h.bridge.lock() { bridge.disconnect(); }
}

/// Wysyła jeden request. `out_len` działa jednocześnie jako rozmiar wejściowy
/// bufora i rzeczywista długość odpowiedzi po sukcesie.
#[no_mangle]
pub extern "C" fn rust_bridge_transact(
    handle: *mut RustBridgeHandle,
    operation: u32,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_capacity: usize,
    out_len: *mut usize,
) -> i32 {
    if handle.is_null() || out_len.is_null() { return BridgeStatus::InvalidArgument.as_i32(); }
    if input_len > 0 && input.is_null() { return BridgeStatus::InvalidArgument.as_i32(); }
    if output_capacity > 0 && output.is_null() { return BridgeStatus::InvalidArgument.as_i32(); }
    if input_len > BRIDGE_MAX_MESSAGE { return BridgeStatus::BufferTooSmall.as_i32(); }

    let payload = if input_len == 0 { &[] } else { unsafe { slice::from_raw_parts(input, input_len) } };
    let operation = match operation {
        0x0001 => Operation::Handshake, 0x0002 => Operation::QueryVersion,
        0x0010 => Operation::Read, 0x0011 => Operation::Write,
        0x0012 => Operation::Ioctl, 0x0020 => Operation::MapMemory,
        0x0021 => Operation::UnmapMemory, 0x0030 => Operation::Event,
        0x00ff => Operation::Shutdown, _ => return BridgeStatus::InvalidArgument.as_i32(),
    };

    let h = unsafe { &*handle };
    let response = match h.bridge.lock().map_err(|_| BridgeError::Synchronization).and_then(|mut b| b.transact(operation, payload)) {
        Ok(value) => value,
        Err(error) => return error.status().as_i32(),
    };
    unsafe { *out_len = response.len(); }
    if response.len() > output_capacity { return BridgeStatus::BufferTooSmall.as_i32(); }
    if !response.is_empty() { unsafe { ptr::copy_nonoverlapping(response.as_ptr(), output, response.len()); } }
    BridgeStatus::Ok.as_i32()
}

/* End of win_kernel_bridge.rs */
