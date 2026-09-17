extern "C" {
    fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32;

    fn cl_compile_source(src: *const u8, len: usize, ar: *mut u8,
                         err_line: *mut u32,
                         err_msg: *mut *const u8) -> *mut u8;

    fn cl_vm_init(vm: *mut u8, prog: *mut u8);
    fn cl_bridge_init(vm: *mut u8, ring: u8) -> i32;
    fn cl_vm_run(vm: *mut u8) -> i32;
}

static mut SRC: [u8; 65536] = [0; 65536];

#[repr(align(16))]
struct ArenaBuf([u8; 196608]);
static mut ARENA: ArenaBuf = ArenaBuf([0; 196608]);

#[repr(align(16))]
struct ProgBuf([u8; 65536]);
static mut PROG: ProgBuf = ProgBuf([0; 65536]);

#[repr(align(16))]
struct VmBuf([u8; 98304]);
static mut VM: VmBuf = VmBuf([0; 98304]);

pub fn run(path: &str) -> i32 {
    unsafe {
        let n = k_fs_read(path.as_ptr(), SRC.as_mut_ptr(), 65535);

        if (n as isize) <= 0 {
            return -1;
        }

        let mut el = 0u32;
        let mut em: *const u8 = core::ptr::null();

        let prog = cl_compile_source(SRC.as_ptr(), n as usize,
                                     ARENA.0.as_mut_ptr(), &mut el, &mut em);

        if prog.is_null() {
            return -2;
        }

        cl_vm_init(VM.0.as_mut_ptr(), prog);
        cl_bridge_init(VM.0.as_mut_ptr(), 3);

        cl_vm_run(VM.0.as_mut_ptr())
    }
}