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

// ── the video class opcode map ───────────────────────────────────────────────
//
// The class number is 5 (`OpClass::Video`) and the operation number is the low
// byte. The numbers below are not one list in one place, so they are gathered
// here; anything not named is refused rather than falling through.
//
//   0  alias for FB_INFO          (see below)
//   1  FB_INFO                   uspace::KernelClient::acquire_framebuffer
//   2  —                          free
//   3  HDMI_INIT                  kernel/src/hdmi/aut.rs
//   4  HDMI_FILL
//   5  HDMI_POLL
//   6  HDMI_CAPS
//   7  MODE_GET
//   8  MODE_LIST
//   9  MODE_SET
//  10  GRANT_FB
//  11  REVOKE_FB
//  12  HDMI_ACQUIRE
//  13  HDMI_RELEASE
//
// The HDMI operations are reached through `hdmi::bridge::hdmi_call`, which is
// wired separately from this dispatcher; they are listed so the numbering
// cannot be reused by accident.

/// Framebuffer info: the canonical operation number.
///
/// This is 1 because that is what the only real client sends
/// (`uspace::KernelClient` builds `(5 << 8) | 1`). An earlier version of this
/// file answered only operation 0, which broke `acquire_framebuffer` — the
/// client got `status = -1` and no framebuffer. The number has to match the
/// client that actually exists, not the one that seems tidy.
const VIDEO_FB_INFO: u32 = 1;

/// Operation 0 was never assigned to anything; accepted as an alias for
/// [`VIDEO_FB_INFO`] so a client built against the earlier numbering still
/// works. Two spellings of one operation is a nuisance, but a dangling client
/// is worse.
const VIDEO_FB_INFO_ALIAS: u32 = 0;

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
            VIDEO_FB_INFO | VIDEO_FB_INFO_ALIAS => {
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

