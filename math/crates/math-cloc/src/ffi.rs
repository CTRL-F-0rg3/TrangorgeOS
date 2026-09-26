use crate::ClocTime;
use core::ptr;

/// Error codes for FFI boundary.
pub const MATH_CLOC_SUCCESS: i32 = 0;
pub const MATH_CLOC_ERR_NULL_PTR: i32 = -1;
pub const MATH_CLOC_ERR_OVERFLOW: i32 = -2;
pub const MATH_CLOC_ERR_TIME_REWIND: i32 = -3;

/// FFI wrapper for checked addition.
#[no_mangle]
pub unsafe extern "C" fn math_cloc_add(
    a: ClocTime,
    b: ClocTime,
    out: *mut ClocTime,
) -> i32 {
    if out.is_null() {
        return MATH_CLOC_ERR_NULL_PTR;
    }

    match a.checked_add(b) {
        Some(result) => {
            ptr::write_unaligned(out, result);
            MATH_CLOC_SUCCESS
        }
        None => MATH_CLOC_ERR_OVERFLOW,
    }
}

/// FFI wrapper for monotonicity validation.
#[no_mangle]
pub extern "C" fn math_cloc_validate_monotonic(
    current: ClocTime,
    previous: ClocTime,
) -> i32 {
    if current.is_monotonic(previous) {
        MATH_CLOC_SUCCESS
    } else {
        MATH_CLOC_ERR_TIME_REWIND
    }
}