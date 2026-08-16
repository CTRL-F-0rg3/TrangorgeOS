pub const TRB_NORMAL: u32 = 1;
pub const TRB_SETUP: u32 = 2;
pub const TRB_DATA: u32 = 3;
pub const TRB_STATUS: u32 = 4;
pub const TRB_LINK: u32 = 6;
pub const TRB_EVENT_DATA: u32 = 7;
pub const TRB_NOOP_TR: u32 = 8;
pub const TRB_ENABLE_SLOT: u32 = 9;
pub const TRB_DISABLE_SLOT: u32 = 10;
pub const TRB_ADDRESS_DEVICE: u32 = 11;
pub const TRB_CONFIGURE_EP: u32 = 12;
pub const TRB_EVALUATE_CTX: u32 = 13;
pub const TRB_RESET_EP: u32 = 14;
pub const TRB_STOP_EP: u32 = 15;
pub const TRB_SET_DEQUEUE: u32 = 16;
pub const TRB_RESET_DEV: u32 = 17;
pub const TRB_NOOP_CMD: u32 = 23;
pub const TRB_TRANSFER_EVENT: u32 = 32;
pub const TRB_CMD_COMPLETION: u32 = 33;
pub const TRB_PORT_STATUS: u32 = 34;
pub const TRB_HC_EVENT: u32 = 35;

pub const CC_SUCCESS: u8 = 1;

const ADDR_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Trb {
    pub param: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn typ(&self) -> u32 {
        (self.control >> 10) & 0x3F
    }

    pub fn cycle(&self) -> bool {
        self.control & 1 != 0
    }

    pub fn completion_code(&self) -> u8 {
        (self.status >> 24) as u8
    }

    pub fn slot_id(&self) -> u8 {
        (self.control >> 24) as u8
    }

    pub fn ep_id(&self) -> u8 {
        ((self.control >> 16) & 0x1F) as u8
    }

    pub fn transfer_len(&self) -> u32 {
        self.status & 0x00FF_FFFF
    }

    pub fn link(addr: u64) -> Self {
        Trb {
            param: addr & ADDR_MASK,
            status: 0,
            control: (TRB_LINK << 10) | (1 << 1),
        }
    }

    pub fn enable_slot() -> Self {
        Trb { control: TRB_ENABLE_SLOT << 10, ..Default::default() }
    }

    pub fn address_device(slot: u8, ctx_phys: u64) -> Self {
        Trb {
            param: ctx_phys & ADDR_MASK,
            control: (TRB_ADDRESS_DEVICE << 10) | ((slot as u32) << 24),
        }
    }

    pub fn configure_ep(slot: u8, ctx_phys: u64) -> Self {
        Trb {
            param: ctx_phys & ADDR_MASK,
            control: (TRB_CONFIGURE_EP << 10) | ((slot as u32) << 24),
        }
    }

    pub fn noop_cmd() -> Self {
        Trb { control: TRB_NOOP_CMD << 10, ..Default::default() }
    }

    pub fn setup_stage(raw_setup: u64, trt: u32) -> Self {
        Trb {
            param: raw_setup,
            control: (TRB_SETUP << 10) | (1 << 6) | ((trt & 3) << 16),
        }
    }

    pub fn data_stage(addr: u64, len: u32, dir_in: bool) -> Self {
        Trb {
            param: addr & ADDR_MASK,
            status: len & 0xFFFF,
            control: (TRB_DATA << 10) | ((dir_in as u32) << 16),
        }
    }

    pub fn status_stage(dir_in: bool) -> Self {
        Trb {
            control: (TRB_STATUS << 10) | ((dir_in as u32) << 16) | (1 << 5),
        }
    }
}
pub fn evaluate_ctx(slot: u8, ctx_phys: u64) -> Self {
    Trb {
        param: ctx_phys & ADDR_MASK,
        control: (TRB_EVALUATE_CTX << 10) | ((slot as u32) << 24),
    }
}
pub fn pack_setup(bm_request: u8, b_request: u8, value: u16,
                  index: u16, length: u16) -> u64 {
    (bm_request as u64)
        | ((b_request as u64) << 8)
        | ((value as u64) << 16)
        | ((index as u64) << 32)
        | ((length as u64) << 48)
}