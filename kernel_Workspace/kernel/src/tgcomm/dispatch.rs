//! Self-contained dispatch: maps a `tg_comm` `CommMsg` onto kernel work.
//!
//! The legacy `driverspaceinit::init::service` dispatcher is a stale file that
//! no longer compiles (it references removed `VGPU_*`/`*_call` helpers), so this
//! module dispatches directly against active kernel services. The transport —
//! shared-memory `tg_comm` rings plus the capability authorization gate — lives
//! in `bridge.rs`; this module is only the "what to do once authorized" part.

use tg_comm::CommMsg;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

// `tg_comm` opcode classes (see lib/src/opcodes.rs).
const CLS_SYS: u32 = 0;
const CLS_VIDEO: u32 = 5;

/// Framebuffer info, within the video class.
const VIDEO_FB_INFO: u32 = 0;

/// Dispatch an authorized request, filling `reply` with the result.
pub fn handle(msg: &CommMsg, reply: &mut CommMsg) {
    let cls = (msg.opcode >> 8) as u32;
    let op = (msg.opcode & 0xFF) as u32;

    match cls {
        CLS_SYS => match op {
            // Identify: handshake marker + protocol version.
            1 => {
                reply.status = 0;
                reply.a0 = 0x5452_474F_535F_4F4B; // "TRGOS_OK"
                reply.a1 = tg_comm::COMM_VERSION as u64;
            }
            // Log: emit a fixed kernel-console line (round-trip check).
            10 => unsafe {
                kprintf(b"[tgcomm] request layer=%d op=%d\n\0".as_ptr(),
                       msg.layer as i32, op as i32);
                reply.status = 0;
            },
            _ => {
                reply.status = -1;
            }
        },
        CLS_VIDEO => match op {
            // Framebuffer info. The reply carries the whole
            // `FramebufferDesc`, in the packing below.
            VIDEO_FB_INFO => {
                let desc = crate::gfx::console::fb_desc();

                if !desc.is_usable() {
                    // There is no framebuffer to describe. A success reply here
                    // would let a client map physical address zero, which is the
                    // configuration space at best and an arbitrary device at
                    // worst, so this is a failure rather than zeros.
                    reply.status = -1;
                } else {
                    // `GfxFbInfo` in `kapi-abi` is 0x0300, while `tg_comm` puts
                    // the class in the high byte, so the two numberings cannot
                    // both be right. `tg_comm` is the transport actually in use
                    // and `OpClass::Video` is 5, so that is what travels here;
                    // `ds-fw-gpu` maps 0x0300 onto the same descriptor.
                    reply.status = 0;
                    reply.a0 = ((desc.width as u64) << 16) | desc.height as u64;
                    // The stride in *bytes*, as the descriptor defines it. The
                    // old reply divided by four, which described the same
                    // framebuffer in different units on each side.
                    reply.a1 = desc.stride_px as u64;
                    reply.a2 = desc.phys;
                    // `CommMsg` has a fourth argument register, so the format and
                    // the flip do not have to be packed into the others: `a3`
                    // carries the format in its low 32 bits and the flip flag in
                    // its high 32. One field, one place.
                    reply.a3 = (desc.format as u64) | ((desc.flipped as u64) << 32);
                }
            }
            _ => {
                reply.status = -1;
            }
        },
        _ => {
            reply.status = -1;
        }
    }
}

