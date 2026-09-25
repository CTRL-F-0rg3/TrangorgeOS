// tgcomm.odin — Odin bindings for the TrangorgeOS strict shared-memory
// communication protocol.
//
// These are thin `foreign` declarations that call the Rust core
// (`../lib`, exported as `libtg_comm.a`). The message layout is public; the
// capability table, rings and the authorization gate stay opaque and are only
// manipulated through the `tgcomm_*` procedures below.
package tgcomm

// ---------------------------------------------------------------------------
// Protocol constants (mirror lib/src/consts.rs)
// ---------------------------------------------------------------------------
MAGIC              :: 0x5447434D
VERSION            :: 1
MSG_SIZE           :: 64
RING_CTRL          :: 16
RING_SLOTS_DEFAULT :: 256
CAP_TABLE_SIZE     :: 512
CAP_TABLE_BYTES    :: 16384

// ---------------------------------------------------------------------------
// Message kind
// ---------------------------------------------------------------------------
Msg_Kind :: enum u8 {
    Request      = 0,
    Reply        = 1,
    Event        = 2,
    Notification = 3,
    Cap_Transfer = 4,
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------
Layer :: enum u8 {
    Kernel           = 0,
    Driverspace      = 1,
    Userspace        = 2,
    Manager          = 3,
    User_Driver_Space = 4,
}

// ---------------------------------------------------------------------------
// Object types
// ---------------------------------------------------------------------------
Object_Type :: enum u8 {
    Memory       = 0,
    Endpoint     = 1,
    Notification = 2,
    Channel      = 3,
    Shmem_Region = 4,
    Device       = 5,
    Irq          = 6,
}

// ---------------------------------------------------------------------------
// Rights bitmask
// ---------------------------------------------------------------------------
RIGHT_READ     :: 1 << 0
RIGHT_WRITE    :: 1 << 1
RIGHT_EXEC     :: 1 << 2
RIGHT_MAP      :: 1 << 3
RIGHT_GRANT    :: 1 << 4
RIGHT_TRANSFER :: 1 << 5
RIGHT_SEND     :: 1 << 6
RIGHT_RECV     :: 1 << 7
RIGHT_CALL     :: 1 << 8
RIGHT_MANAGE   :: 1 << 9

// ---------------------------------------------------------------------------
// Opcode classes
// ---------------------------------------------------------------------------
Op_Class :: enum u8 {
    Sys   = 0,
    Mem   = 1,
    Cap   = 2,
    Ipc   = 3,
    Shmem = 4,
    Video = 5,
    Audio = 6,
    Input = 7,
    Block = 8,
    Net   = 9,
    Pci   = 10,
    Vgpu  = 11,
    Fs    = 12,
}

opcode :: proc(cls: Op_Class, op: u16) -> u32 {
    return (u32(cls) << 8) | u32(op & 0xFF)
}

// ---------------------------------------------------------------------------
// Authorize result (mirror filter.rs)
// ---------------------------------------------------------------------------
Result :: enum i32 {
    Forwarded           = 0,
    Malformed           = 1,
    Unknown_Op          = 2,
    Route_Denied        = 3,
    No_Capability       = 4,
    Rights_Insufficient = 5,
    Type_Mismatch       = 6,
}

// ---------------------------------------------------------------------------
// The fixed-size wire message (mirror lib/src/wire.rs, 64 bytes)
// ---------------------------------------------------------------------------
TgComm_Msg :: struct #packed {
    magic:    u32,
    version:  u8,
    kind:     u8,
    layer:    u8,
    target:   u8,
    opcode:   u32,
    cap:      u32,
    seq:      u32,
    status:   i32,
    a0, a1, a2, a3: u64,
    reserved: [8]u8,
}

#assert(size_of(TgComm_Msg) == 64)

// ---------------------------------------------------------------------------
// Opaque capability table (reserve storage; real layout lives in Rust)
// ---------------------------------------------------------------------------
TgComm_Cap_Table :: struct #align(16) {
    storage: [CAP_TABLE_BYTES]u8,
}

// ---------------------------------------------------------------------------
// Stable C ABI (provided by libtg_comm.a). These call into Rust.
// ---------------------------------------------------------------------------
foreign tg_comm {
    tgcomm_version        :: proc() -> u32 ---
    tgcomm_msg_size       :: proc() -> u32 ---
    tgcomm_ring_size      :: proc(slots: u32) -> uint ---
    tgcomm_cap_table_size :: proc() -> uint ---

    tgcomm_msg_init       :: proc(msg: ^TgComm_Msg) ---
    tgcomm_msg_valid      :: proc(msg: ^TgComm_Msg) -> i32 ---

    tgcomm_authorize      :: proc(table: rawptr, msg: ^TgComm_Msg) -> i32 ---

    tgcomm_ring_init      :: proc(ring: rawptr, slots: u32) ---
    tgcomm_ring_push      :: proc(ring: rawptr, msg: ^TgComm_Msg) -> i32 ---
    tgcomm_ring_pop       :: proc(ring: rawptr, out: ^TgComm_Msg) -> i32 ---
    tgcomm_ring_available :: proc(ring: rawptr) -> u32 ---

    tgcomm_cap_table_init :: proc(table: rawptr) ---
    tgcomm_cap_insert     :: proc(table: rawptr, cap: u32, obj_id: u64,
                                  obj_type: u8, rights: u32) -> i32 ---
    tgcomm_cap_check      :: proc(table: rawptr, cap: u32,
                                  required_rights: u32) -> i32 ---
    tgcomm_cap_remove     :: proc(table: rawptr, cap: u32) -> i32 ---
}

// ---------------------------------------------------------------------------
// Convenience helpers (Odin-side, still delegate to the Rust core)
// ---------------------------------------------------------------------------
ring_bytes :: proc(slots: u32) -> uint {
    return tgcomm_ring_size(slots)
}

build_request :: proc(msg: ^TgComm_Msg, layer: Layer, target: Layer,
                      op: u32, cap: u32) {
    tgcomm_msg_init(msg)
    msg.kind   = u8(Msg_Kind.Request)
    msg.layer  = u8(layer)
    msg.target = u8(target)
    msg.opcode = op
    msg.cap    = cap
}
