#[macro_export]
macro_rules! align_up {
    ($val:expr, $align:expr) => {
        ($val + ($align - 1)) & !($align - 1)
    };
}

#[macro_export]
macro_rules! align_down {
    ($val:expr, $align:expr) => {
        $val & !($align - 1)
    };
}

#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(core::ptr::null::<$ty>())).$field as *const _ as usize }
    };
}

#[macro_export]
macro_rules! container_of {
    ($ptr:expr, $ty:ty, $field:ident) => {
        ($ptr as usize - $crate::offset_of!($ty, $field)) as *mut $ty
    };
}

#[macro_export]
macro_rules! bit {
    ($n:expr) => {
        (1 << $n)
    };
}