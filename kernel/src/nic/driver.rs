use crate::nic::device::NetworkDevice;

const MAX_DEVICES: usize = 4;

pub struct DriverRegistry<'a> {
    devices: [Option<&'a mut dyn NetworkDevice>; MAX_DEVICES],
    count: usize,
}

impl<'a> DriverRegistry<'a> {
    pub fn new() -> Self {
        DriverRegistry {
            devices: [None, None, None, None],
            count: 0,
        }
    }

    pub fn register(&mut self, device: &'a mut dyn NetworkDevice) -> bool {
        if self.count >= MAX_DEVICES {
            return false;
        }
        self.devices[self.count] = Some(device);
        self.count += 1;
        true
    }

    pub fn device(&mut self, index: usize) -> Option<&mut (dyn NetworkDevice + 'a)> {
        self.devices.get_mut(index)?.as_deref_mut()
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<'a> Default for DriverRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}
