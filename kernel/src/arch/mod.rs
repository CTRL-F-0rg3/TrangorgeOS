#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as imp;

#[cfg(target_arch = "riscv64")]
pub use riscv64 as imp;

pub fn init() {
    imp::init();
}


pub fn hlt_loop() -> ! {
    imp::hlt_loop()
}


pub fn now() -> u64 {
    imp::now()
}


pub fn current_cpu() -> usize {
    imp::current_cpu()
}