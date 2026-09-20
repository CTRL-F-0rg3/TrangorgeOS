package abi

import "util"

Opcode :: enum u32 {
    Sys_Ping          = 0x00000001,
    Sys_GetInfo       = 0x00000002,
    Sys_Shutdown      = 0x000000FF,
    Proc_Create       = 0x01000001,
    Proc_Exit         = 0x01000002,
    Proc_Kill         = 0x01000003,
    Proc_Wait         = 0x01000004,
    Mem_Alloc         = 0x02000001,
    Mem_Free          = 0x02000002,
    Mem_Protect       = 0x02000003,
    Ipc_PortCreate    = 0x03000001,
    Ipc_Send          = 0x03000002,
    Ipc_Recv          = 0x03000003,
    Ipc_ShareMem      = 0x03000004,
    FS_Open           = 0x04000001,
    FS_Read           = 0x04000002,
    FS_Write          = 0x04000003,
    FS_Close          = 0x04000004,
    _Reserved         = 0xFFFFFFFF,
}

is_proc_opcode :: proc(op: Opcode) -> bool {
    return (cast(u32, op) & 0xFF000000) == 0x01000000
}

is_mem_opcode :: proc(op: Opcode) -> bool {
    return (cast(u32, op) & 0xFF000000) == 0x02000000
}

is_ipc_opcode :: proc(op: Opcode) -> bool {
    return (cast(u32, op) & 0xFF000000) == 0x03000000
}

is_fs_opcode :: proc(op: Opcode) -> bool {
    return (cast(u32, op) & 0xFF000000) == 0x04000000
}