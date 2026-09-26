const std = @import("std");

pub const ClocTime = extern struct {
    secs: u64,
    nanos: u32,
};

pub const Error = error{
    NullPointer,
    Overflow,
    TimeRewind,
};

// Direct FFI declarations
extern fn math_cloc_add(a: ClocTime, b: ClocTime, out: *ClocTime) i32;
extern fn math_cloc_validate_monotonic(current: ClocTime, previous: ClocTime) i32;

pub fn add(a: ClocTime, b: ClocTime) Error!ClocTime {
    var out: ClocTime = undefined;
    const status = math_cloc_add(a, b, &out);

    return switch (status) {
        0 => out,
        -1 => Error.NullPointer,
        -2 => Error.Overflow,
        else => unreachable,
    };
}

pub fn validateMonotonic(current: ClocTime, previous: ClocTime) Error!void {
    const status = math_cloc_validate_monotonic(current, previous);
    if (status != 0) return Error.TimeRewind;
}
