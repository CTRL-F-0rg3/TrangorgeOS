use core::sync::atomic::{AtomicBool, Ordering};

static OOM_HANDLER_ACTIVE: AtomicBool = AtomicBool::new(false);

pub trait OomHandler: Fn() -> ! + Send + Sync {}
impl<T: Fn() -> ! + Send + Sync> OomHandler for T {}

static mut HANDLER: Option<&'static dyn OomHandler> = None;

pub fn set_handler(handler: &'static dyn OomHandler) {
    unsafe { HANDLER = Some(handler) };
}

pub fn invoke() -> ! {
    if OOM_HANDLER_ACTIVE.swap(true, Ordering::SeqCst) {
        loop { core::hint::spin_loop(); }
    }
    unsafe {
        if let Some(h) = HANDLER {
            h();
        }
    }
    panic!("OOM");
}