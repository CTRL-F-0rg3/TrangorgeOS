package kernel_if

import "util"

SYS_DEBUG_PRINT :: u64(1)
SYS_PAGE_ALLOC  :: u64(2)
SYS_IPC_RECV    :: u64(3)

#private syscall_raw :: proc(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    ret: u64
    #asm {
        mov rax, num
        mov rdi, arg1
        mov rsi, arg2
        mov rdx, arg3
        syscall
        mov ret, rax
    }
    return ret
}

pub debug_print :: proc(msg: string) -> util.Errno {
    res := syscall_raw(SYS_DEBUG_PRINT, cast(u64, msg.ptr), cast(u64, len(msg)), 0)
    return cast(util.Errno, res)
}

pub page_alloc :: proc(vaddr: util.VAddr, size: util.Size) -> util.Errno {
    res := syscall_raw(SYS_PAGE_ALLOC, cast(u64, vaddr), cast(u64, size), 0)
    return cast(util.Errno, res)
}

pub ipc_recv :: proc(port_handle: util.Handle, buf_ptr: u64, buf_sz: u64) -> u64 {
    return syscall_raw(SYS_IPC_RECV, cast(u64, port_handle), buf_ptr, buf_sz)
}