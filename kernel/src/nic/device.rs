use crate::nic::{error::NetworkError, types::MacAddress};

/// Wynik jednego lekkiego przebiegu pętli obsługi urządzenia.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PollResult {
    pub tx_completed: u16,
    pub rx_available: u16,
    pub device_needs_reset: bool,
}

/// Niezmienny widok ramki gotowej do wysłania.
#[derive(Debug, Clone, Copy)]
pub struct TxFrame<'a> {
    pub bytes: &'a [u8],
}

impl<'a> TxFrame<'a> {
    #[inline]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

/// Ramka RX wraz z identyfikatorem bufora puli.
///
/// Identyfikator jest przekazywany z powrotem przez `recycle_rx`, dzięki czemu
/// sterownik może natychmiast udostępnić ten sam bufor urządzeniu.
#[derive(Debug)]
pub struct RxFrame<'a> {
    pub buffer_id: u16,
    pub bytes: &'a [u8],
}

/// Minimalny kontrakt dla dowolnego NIC.
///
/// Wszystkie funkcje są jawne i nie uruchamiają pracy w tle; jądro decyduje,
/// kiedy wywołać `poll`, więc profil zużycia CPU jest przewidywalny.
pub trait NetworkDevice {
    fn init(&mut self) -> Result<(), NetworkError>;
    fn mac_address(&self) -> MacAddress;
    fn mtu(&self) -> usize;

    /// Kolejkuje ramkę TX. Dane muszą pozostać ważne do raportu ukończenia TX.
    fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError>;

    /// Odbiera użyte deskryptory. Należy wywoływać okresowo lub po IRQ.
    fn poll(&mut self) -> Result<PollResult, NetworkError>;

    /// Zwraca jedną odebraną ramkę, jeśli `poll()` wykrył nowe dane.
    fn take_rx(&mut self) -> Option<RxFrame<'_>>;

    /// Zwraca bufor RX do puli i udostępnia go urządzeniu.
    fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError>;
}
