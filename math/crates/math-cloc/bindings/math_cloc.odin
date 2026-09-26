package math_cloc

import "core:c"

ClocTime :: struct {
    secs:  u64,
    nanos: u32,
}

foreign "math_cloc" {
    @(link_name = "math_cloc_add")
    add :: proc "c" (a: ClocTime, b: ClocTime, out: ^ClocTime) -> i32 ---

    @(link_name = "math_cloc_validate_monotonic")
    validate_monotonic :: proc "c" (current: ClocTime, previous: ClocTime) -> i32 ---
}

// Idiomatic Odin wrapper
safe_add :: proc(a, b: ClocTime) -> (ClocTime, bool) {
    out: ClocTime
    status := add(a, b, &out)
    return out, status == 0
}