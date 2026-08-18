use core::fmt;

/// Sześciobajtowy adres Ethernet. Typ nie wykonuje żadnej alokacji.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const ZERO: Self = Self([0; 6]);
    pub const BROADCAST: Self = Self([0xff; 6]);

    #[inline]
    pub const fn is_broadcast(self) -> bool {
        self.0[0] == 0xff
            && self.0[1] == 0xff
            && self.0[2] == 0xff
            && self.0[3] == 0xff
            && self.0[4] == 0xff
            && self.0[5] == 0xff
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0[0] == 0
            && self.0[1] == 0
            && self.0[2] == 0
            && self.0[3] == 0
            && self.0[4] == 0
            && self.0[5] == 0
    }

    #[inline]
    pub const fn is_multicast(self) -> bool {
        self.0[0] & 1 == 1
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Adres IPv4 zapisany w kolejności sieciowej.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    pub const LIMITED_BROADCAST: Self = Self([255, 255, 255, 255]);

    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    #[inline]
    pub const fn as_u32_be(self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    #[inline]
    pub const fn is_unspecified(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    #[inline]
    pub const fn is_limited_broadcast(self) -> bool {
        self.0[0] == 255 && self.0[1] == 255 && self.0[2] == 255 && self.0[3] == 255
    }

    /// Zwraca prawdę, gdy `other` jest osiągalny bezpośrednio na Ethernet.
    #[inline]
    pub const fn is_in_subnet(self, other: Self, netmask: Self) -> bool {
        (self.as_u32_be() & netmask.as_u32_be()) == (other.as_u32_be() & netmask.as_u32_be())
    }
}

impl fmt::Debug for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subnet_check_uses_mask() {
        let local = Ipv4Address::new(192, 168, 10, 3);
        let mask = Ipv4Address::new(255, 255, 255, 0);
        assert!(local.is_in_subnet(Ipv4Address::new(192, 168, 10, 99), mask));
        assert!(!local.is_in_subnet(Ipv4Address::new(192, 168, 11, 1), mask));
    }
}
