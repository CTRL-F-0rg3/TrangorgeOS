use kstd_base::Status;

extern "C" {
    fn syscall_dispatch(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64;
}

pub fn invoke(num: u64, args: [u64; 4]) -> Result<u64, Status> {
    let ret = unsafe {
        syscall_dispatch(num, args[0], args[1], args[2], args[3])
    };
    
    if ret < 0 {
        Err(Status::from((-ret) as u32))
    } else {
        Ok(ret as u64)
    }
}

#[macro_export]
macro_rules! syscall {
    ($num:expr) => {
        $crate::syscalls::invoke($num, [0; 4])
    };
    ($num:expr, $a1:expr) => {
        $crate::syscalls::invoke($num, [$a1 as u64, 0, 0, 0])
    };
    ($num:expr, $a1:expr, $a2:expr) => {
        $crate::syscalls::invoke($num, [$a1 as u64, $a2 as u64, 0, 0])
    };
}