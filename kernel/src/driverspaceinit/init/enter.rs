use super::super::abi::abi::*;
use super::super::abi::src::RingView;
use super::init;
use super::initabi::*;
use super::initcommand::DsError;

#[naked]
extern "C" fn to_ds(sw: *const u8, entry: u64, stack: u64, arg: u64) {
    core::arch::asm!(
        "mov [rdi + 24], rsp",
        "mov rsp, rdx",
        "mov cr3, [rdi + 8]",
        "mov rdi, rcx",
        "jmp rsi",
        options(naked)
    );
}

#[naked]
extern "C" fn kernel_resume() {
    core::arch::asm!("ret", options(naked));
}

#[naked]
pub extern "C" fn ds_yield() {
    core::arch::asm!(
        "movabs rax, {sw}",
        "mov cr3, [rax + 0]",
        "mov rsp, [rax + 24]",
        "jmp [rax + 16]",
        sw = const DS_SWITCH_VA,
        options(naked)
    );
}

extern "C" fn ds_stub_entry(params_va: u64) {
    unsafe {
        let params = &*(params_va as *const DsInitParams);

        let scratch = DS_SCRATCH_VA as *mut u8;
        let msg = b"hello from driverspace";
        core::ptr::copy_nonoverlapping(msg.as_ptr(), scratch, msg.len());

        let rv = RingView::new(DS_K2D_VA as *mut u8);

        let _ = rv.push(&DsMsg {
            id: 1,
            cmd: DsCmd::Log as u32,
            flags: 0,
            arg0: msg.len() as u64,
            arg1: params.magic,
            arg2: 0,
            status: 0,
            pad: 0,
        });
    }

    ds_yield();
}

pub fn enter() -> Result<(), DsError> {
    let sw = init::switch_view().ok_or(DsError::NotPrepared)?;
    let ds_cr3 = init::ds_cr3().ok_or(DsError::NotPrepared)?;

    unsafe {
        let p = sw as *mut u64;

        p.add(0).write_volatile(init::kernel_cr3());
        p.add(1).write_volatile(ds_cr3);
        p.add(2).write_volatile(kernel_resume as u64);

        to_ds(sw,
              ds_stub_entry as u64,
              DS_STACK_VA + DS_STACK_SIZE,
              DS_INIT_PARAMS_VA);
    }

    Ok(())
}