/*
 * tgcomm.c — thin C-side helpers for the TrangorgeOS communication protocol.
 *
 * The actual protocol engine (wire format, capability table, shared-memory
 * ring and the authorization gate) is implemented once in Rust and exported
 * through the `tgcomm_*` C ABI in `libtg_comm.a`. This translation unit only
 * adds a couple of convenience helpers that call that ABI — it never
 * re-implements the protocol.
 */
#include "tgcomm.h"

uint32_t tgcomm_ring_bytes(uint32_t slots) {
    /* Forward to the Rust core so there is a single source of truth. */
    return (uint32_t)tgcomm_ring_size(slots);
}
