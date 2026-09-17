#[derive(Clone, Copy, Default)]
pub struct DeviceDesc {
    pub max_packet0: u8,
    pub vendor: u16,
    pub product: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub num_configs: u8,
}

#[derive(Clone, Copy, Default)]
pub struct InterfaceDesc {
    pub number: u8,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub num_eps: u8,
}

#[derive(Clone, Copy, Default)]
pub struct EndpointDesc {
    pub address: u8,
    pub attributes: u8,
    pub max_packet: u16,
    pub interval: u8,
}

pub fn parse_device(buf: &[u8]) -> Option<DeviceDesc> {
    if buf.len() < 18 || buf[1] != 1 {
        return None;
    }

    Some(DeviceDesc {
        max_packet0: buf[7],
        vendor: u16::from_le_bytes([buf[8], buf[9]]),
        product: u16::from_le_bytes([buf[10], buf[11]]),
        class: buf[4],
        subclass: buf[5],
        protocol: buf[6],
        num_configs: buf[17],
    })
}

pub fn parse_config(buf: &[u8],
                    ifaces: &mut [InterfaceDesc],
                    eps: &mut [EndpointDesc])
                    -> Option<(u8, u16, usize, usize)> {
    if buf.len() < 9 || buf[1] != 2 {
        return None;
    }

    let total = u16::from_le_bytes([buf[2], buf[3]]);
    let config_value = buf[5];

    let mut off = 9usize;
    let mut ni = 0usize;
    let mut ne = 0usize;

    while off + 2 <= buf.len().min(total as usize) {
        let len = buf[off] as usize;
        let typ = buf[off + 1];

        if len < 2 {
            break;
        }

        match typ {
            4 => {
                if off + 9 <= buf.len() && ni < ifaces.len() {
                    ifaces[ni] = InterfaceDesc {
                        number: buf[off + 2],
                        class: buf[off + 5],
                        subclass: buf[off + 6],
                        protocol: buf[off + 7],
                        num_eps: buf[off + 8],
                    };
                    ni += 1;
                }
            }
            5 => {
                if off + 7 <= buf.len() && ne < eps.len() {
                    eps[ne] = EndpointDesc {
                        address: buf[off + 2],
                        attributes: buf[off + 3],
                        max_packet: u16::from_le_bytes([buf[off + 4], buf[off + 5]]),
                        interval: buf[off + 6],
                    };
                    ne += 1;
                }
            }
            _ => {}
        }

        off += len;
    }

    Some((config_value, total, ni, ne))
}