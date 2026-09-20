pub mod string;
pub mod math;

pub use string::{cstr_len, cstr_to_slice, cstr_to_str, StrBuffer};
pub use math::{align_up, align_down, is_aligned, min, max, div_ceil};