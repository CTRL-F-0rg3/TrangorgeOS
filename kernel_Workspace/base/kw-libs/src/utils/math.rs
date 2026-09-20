// Alignment and basic math helpers.

pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

pub const fn align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}

pub const fn is_aligned(val: usize, align: usize) -> bool {
    val & (align - 1) == 0
}

pub const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

pub const fn max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

pub const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}