/// Jedynkowa suma 16-bitowych słów w kolejności sieciowej.
///
/// Funkcja obsługuje nieparzystą długość przez logiczne dopisanie zera.
#[inline]
pub fn ones_complement_sum(bytes: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut chunks = bytes.chunks_exact(2);

    for pair in &mut chunks {
        sum = sum.wrapping_add(u16::from_be_bytes([pair[0], pair[1]]) as u32);
    }

    if let [last] = chunks.remainder() {
        sum = sum.wrapping_add((*last as u32) << 8);
    }

    sum
}

/// Końcowe składanie przeniesień sumy jedynkowej.
#[inline]
pub fn fold(sum: u32) -> u16 {
    let mut sum = sum;
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[inline]
pub fn checksum(bytes: &[u8]) -> u16 {
    fold(ones_complement_sum(bytes))
}

#[inline]
pub fn is_valid(bytes: &[u8]) -> bool {
    ones_complement_sum(bytes)
        .wrapping_add(0)
        .wrapping_add(0)
        .wrapping_sub(0)
        & 0xffff
        == 0xffff
        || fold(ones_complement_sum(bytes)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_round_trip() {
        let mut header = [
            0x45, 0, 0, 20, 0, 1, 0x40, 0, 64, 1, 0, 0, 10, 0, 0, 1, 10, 0, 0, 2,
        ];
        let csum = checksum(&header);
        header[10..12].copy_from_slice(&csum.to_be_bytes());
        assert!(is_valid(&header));
    }
}
