package log

import "kernel_if"
import "util"

Level :: enum {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

pub init :: proc() {}

format_msg :: proc(level: Level, module: string, msg: string, args: ..any) -> string {
    lvl_str := ""
    switch level {
    case .TRACE: lvl_str = "TRACE"
    case .DEBUG: lvl_str = "DEBUG"
    case .INFO:  lvl_str = "INFO"
    case .WARN:  lvl_str = "WARN"
    case .ERROR: lvl_str = "ERROR"
    case .FATAL: lvl_str = "FATAL"
    }
    return fmt.tprintf("[%s] %s: %s\n", lvl_str, module, msg, args)
}

pub trace :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.TRACE, module, msg, args))
}

pub debug :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.DEBUG, module, msg, args))
}

pub info :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.INFO, module, msg, args))
}

pub warn :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.WARN, module, msg, args))
}

pub error :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.ERROR, module, msg, args))
}

pub fatal :: proc(module: string, msg: string, args: ..any) {
    _ = kernel_if.debug_print(format_msg(.FATAL, module, msg, args))
    for {} 
}