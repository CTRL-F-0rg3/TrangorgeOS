pub const SVC_SYS: u32 = 0;
pub const SVC_VIDEO: u32 = 1;
pub const SVC_AUDIO: u32 = 2;
pub const SVC_INPUT: u32 = 3;
pub const SVC_BLOCK: u32 = 4;
pub const SVC_NET: u32 = 5;

pub const fn svc_cmd(class: u32, op: u32) -> u32 {
    (class << 8) | (op & 0xFF)
}

pub const fn svc_class(cmd: u32) -> u32 {
    cmd >> 8
}

pub const fn svc_op(cmd: u32) -> u32 {
    cmd & 0xFF
}

// VIDEO ops
pub const VID_FB_INFO: u32 = 1;
pub const VID_FB_TAKEOVER: u32 = 2;
pub const VID_FB_RELEASE: u32 = 3;

// INPUT ops
pub const IN_KEY_POLL: u32 = 1;

// AUDIO ops
pub const AUD_PLAY: u32 = 1;
pub const AUD_STOP: u32 = 2;
pub const AUD_JACK: u32 = 3;
pub const AUD_AMP: u32 = 4;

// BLOCK ops
pub const BLK_COUNT: u32 = 1;
pub const BLK_READ: u32 = 2;
pub const BLK_WRITE: u32 = 3;