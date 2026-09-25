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
        CLS_VIDEO => {
            // Framebuffer info: (width<<16 | height, stride, physical base).
            let (w, h, stride, phys) = crate::gfx::console::fb_info();
            reply.status = 0;
            reply.a0 = ((w as u64) << 16) | h as u64;
            reply.a1 = stride as u64;
            reply.a2 = phys;
        }
        _ => {
            reply.status = -1;
        }
    }
}

