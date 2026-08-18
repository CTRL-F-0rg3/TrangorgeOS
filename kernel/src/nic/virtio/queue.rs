use crate::nic::error::NetworkError;

pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl Descriptor {
    pub const EMPTY: Self = Self {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    };
}

/// Jednoznaczny identyfikator deskryptora w puli.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct DescriptorId(pub u16);

/// Pula deskryptorów o stałej pojemności.
///
/// Nie jest to pełna virtqueue: transport udostępnia pamięć urządzeniu, a ta
/// struktura zapewnia wyłącznie brak podwójnego przydziału i poprawny odzysk
/// łańcuchów przez kod sterownika.
pub struct DescriptorPool<const N: usize> {
    descriptors: [Descriptor; N],
    free_head: Option<u16>,
    free_count: u16,
}

impl<const N: usize> DescriptorPool<N> {
    pub const fn new() -> Self {
        let mut descriptors = [Descriptor::EMPTY; N];
        let mut index = 0usize;
        while index < N {
            descriptors[index].next = if index + 1 < N { (index + 1) as u16 } else { 0 };
            index += 1;
        }
        Self {
            descriptors,
            free_head: if N == 0 { None } else { Some(0) },
            free_count: N as u16,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub const fn free_count(&self) -> u16 {
        self.free_count
    }

    pub fn allocate(&mut self) -> Result<DescriptorId, NetworkError> {
        let head = self.free_head.ok_or(NetworkError::NoFreeDescriptor)?;
        let index = head as usize;
        if index >= N {
            return Err(NetworkError::BadDescriptor);
        }
        self.free_head = if self.free_count > 1 {
            Some(self.descriptors[index].next)
        } else {
            None
        };
        self.free_count -= 1;
        self.descriptors[index] = Descriptor::EMPTY;
        Ok(DescriptorId(head))
    }

    pub fn configure(
        &mut self,
        id: DescriptorId,
        addr: u64,
        len: u32,
        flags: u16,
        next: Option<DescriptorId>,
    ) -> Result<(), NetworkError> {
        let slot = self
            .descriptors
            .get_mut(id.0 as usize)
            .ok_or(NetworkError::BadDescriptor)?;
        *slot = Descriptor {
            addr,
            len,
            flags: if next.is_some() {
                flags | VIRTQ_DESC_F_NEXT
            } else {
                flags & !VIRTQ_DESC_F_NEXT
            },
            next: next.map(|v| v.0).unwrap_or(0),
        };
        Ok(())
    }

    #[inline]
    pub fn descriptor(&self, id: DescriptorId) -> Result<&Descriptor, NetworkError> {
        self.descriptors
            .get(id.0 as usize)
            .ok_or(NetworkError::BadDescriptor)
    }

    /// Zwalnia pełny łańcuch zwrócony przez urządzenie.
    ///
    /// Limit iteracji zapobiega pętli przy uszkodzonym deskryptorze.
    pub fn release_chain(&mut self, head: DescriptorId) -> Result<(), NetworkError> {
        let mut current = head.0;
        let mut released = 0usize;
        loop {
            if current as usize >= N || released >= N || self.free_count as usize >= N {
                return Err(NetworkError::BadDescriptor);
            }
            let descriptor = self.descriptors[current as usize];
            let has_next = descriptor.flags & VIRTQ_DESC_F_NEXT != 0;
            let next = descriptor.next;

            self.descriptors[current as usize] = Descriptor {
                next: self.free_head.unwrap_or(0),
                ..Descriptor::EMPTY
            };
            self.free_head = Some(current);
            self.free_count += 1;
            released += 1;

            if !has_next {
                return Ok(());
            }
            current = next;
        }
    }
}

impl<const N: usize> Default for DescriptorPool<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Adresy i długości obszarów pamięci split virtqueue przydzielonych przez jądro.
///
/// Ten typ nie tworzy pamięci DMA; dzięki temu nie zakłada modelu alokatora ani
/// mapowania fizycznego. Właściciel platformy przekazuje te dane do adaptera MMIO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueMemory {
    pub descriptor_phys: u64,
    pub driver_phys: u64,
    pub device_phys: u64,
    pub size: u16,
}

/// Kompatybilność z `nic::virtio::queue::self_test` używanym przez kernel.
pub fn self_test() -> Result<&'static str, &'static str> {
    let mut pool = DescriptorPool::<4>::new();
    let first = pool
        .allocate()
        .map_err(|_| "first descriptor allocation failed")?;
    let second = pool
        .allocate()
        .map_err(|_| "second descriptor allocation failed")?;
    pool.configure(first, 0x1000, 12, 0, Some(second))
        .map_err(|_| "first descriptor configuration failed")?;
    pool.configure(second, 0x2000, 30, VIRTQ_DESC_F_WRITE, None)
        .map_err(|_| "second descriptor configuration failed")?;
    pool.release_chain(first)
        .map_err(|_| "descriptor chain release failed")?;
    if pool.free_count() != 4 {
        return Err("descriptor pool did not recover all descriptors");
    }
    Ok("virtqueue descriptor pool verified")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_allocates_and_releases_chain_once() {
        let mut pool = DescriptorPool::<4>::new();
        let first = pool.allocate().unwrap();
        let second = pool.allocate().unwrap();
        pool.configure(first, 0x1000, 12, 0, Some(second)).unwrap();
        pool.configure(second, 0x2000, 30, VIRTQ_DESC_F_WRITE, None)
            .unwrap();
        assert_eq!(pool.free_count(), 2);
        pool.release_chain(first).unwrap();
        assert_eq!(pool.free_count(), 4);
    }

    #[test]
    fn zero_sized_pool_fails_cleanly() {
        assert_eq!(
            DescriptorPool::<0>::new().allocate(),
            Err(NetworkError::NoFreeDescriptor)
        );
    }
}
