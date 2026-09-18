#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::cpu::scheduler::entities::task::{
    TaskStruct, TaskFlags, CpuContext, FxSaveArea, KERNEL_STACK_SIZE,
};
use crate::cpu::scheduler::stats;

const FPU_REG_SIZE: usize = 8;
const FPU_NUM_REGS: usize = 32;
const FPU_SAVE_SIZE: usize = FPU_REG_SIZE * FPU_NUM_REGS;

const SATP_MODE_BARE: u64 = 0;
const SATP_MODE_SV39: u64 = 8;
const SATP_MODE_SV48: u64 = 9;
const SATP_MODE_SV57: u64 = 10;
const SATP_MODE_SHIFT: u64 = 60;
const SATP_ASID_BITS: u64 = 16;
const SATP_PPN_MASK: u64 = (1u64 << 44) - 1;

const SSTATUS_FS_OFF: u64 = 0;
const SSTATUS_FS_INITIAL: u64 = 1;
const SSTATUS_FS_CLEAN: u64 = 2;
const SSTATUS_FS_DIRTY: u64 = 3;
const SSTATUS_FS_SHIFT: u64 = 13;
const SSTATUS_FS_MASK: u64 = 0x3 << SSTATUS_FS_SHIFT;

const SSTATUS_SD: u64 = 1u64 << 63;
const SSTATUS_SPP: u64 = 1u64 << 8;
const SSTATUS_SPIE: u64 = 1u64 << 5;
const SSTATUS_SIE: u64 = 1u64 << 1;

const CSR_SSTATUS: u64 = 0x100;
const CSR_SIE: u64 = 0x104;
const CSR_STVEC: u64 = 0x105;
const CSR_SSCRATCH: u64 = 0x140;
const CSR_SEPC: u64 = 0x141;
const CSR_SCAUSE: u64 = 0x142;
const CSR_STVAL: u64 = 0x143;
const CSR_SIP: u64 = 0x144;
const CSR_SATP: u64 = 0x180;

const CSR_FFLAGS: u64 = 0x001;
const CSR_FRM: u64 = 0x002;
const CSR_FCSR: u64 = 0x003;

const DEBUG_REGISTERS_COUNT: usize = 4;
const TDATA1_DMODE: u64 = 1u64 << 27;
const TDATA1_TYPE_MCONTROL: u64 = 2u64 << 28;
const TDATA1_ACTION_EXCEPTION: u64 = 0;
const TDATA1_ACTION_DEBUG_MODE: u64 = 1;

static SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
static FPU_CONTEXT_SWITCHES: AtomicU64 = AtomicU64::new(0);
static LAZY_FPU_ENABLED: AtomicBool = AtomicBool::new(true);

static mut LAST_FPU_TASK: *mut TaskStruct = core::ptr::null_mut();
static mut FPU_AVAILABLE: bool = false;
static mut FPU_REGS_SIZE: usize = FPU_SAVE_SIZE;

#[inline(always)]
unsafe fn read_satp() -> u64 {
    let satp: u64;
    asm!("csrr {}, satp", out(reg) satp, options(nomem, nostack, preserves_flags));
    satp
}

#[inline(always)]
unsafe fn write_satp(satp: u64) {
    asm!("csrw satp, {}", in(reg) satp, options(nomem, nostack, preserves_flags));
    asm!("sfence.vma", options(nomem, nostack));
}

#[inline(always)]
unsafe fn read_sstatus() -> u64 {
    let sstatus: u64;
    asm!("csrr {}, sstatus", out(reg) sstatus, options(nomem, nostack, preserves_flags));
    sstatus
}

#[inline(always)]
unsafe fn write_sstatus(sstatus: u64) {
    asm!("csrw sstatus, {}", in(reg) sstatus, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_tp() -> u64 {
    let tp: u64;
    asm!("mv {}, tp", out(reg) tp, options(nomem, nostack, preserves_flags));
    tp
}

#[inline(always)]
unsafe fn write_tp(tp: u64) {
    asm!("mv tp, {}", in(reg) tp, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_gp() -> u64 {
    let gp: u64;
    asm!("mv {}, gp", out(reg) gp, options(nomem, nostack, preserves_flags));
    gp
}

#[inline(always)]
unsafe fn write_gp(gp: u64) {
    asm!("mv gp, {}", in(reg) gp, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_sscratch() -> u64 {
    let sscratch: u64;
    asm!("csrr {}, sscratch", out(reg) sscratch, options(nomem, nostack, preserves_flags));
    sscratch
}

#[inline(always)]
unsafe fn write_sscratch(sscratch: u64) {
    asm!("csrw sscratch, {}", in(reg) sscratch, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn sfence_vma(asid: u64, vaddr: u64) {
    if asid == 0 && vaddr == 0 {
        asm!("sfence.vma", options(nomem, nostack));
    } else if asid == 0 {
        asm!("sfence.vma {}, zero", in(reg) vaddr, options(nomem, nostack));
    } else if vaddr == 0 {
        asm!("sfence.vma zero, {}", in(reg) asid, options(nomem, nostack));
    } else {
        asm!("sfence.vma {}, {}", in(reg) vaddr, in(reg) asid, options(nomem, nostack));
    }
}

#[inline(always)]
pub unsafe fn enable_fpu() {
    let mut sstatus = read_sstatus();
    sstatus = (sstatus & !SSTATUS_FS_MASK) | (SSTATUS_FS_INITIAL << SSTATUS_FS_SHIFT);
    write_sstatus(sstatus);
}

#[inline(always)]
pub unsafe fn disable_fpu() {
    let mut sstatus = read_sstatus();
    sstatus = (sstatus & !SSTATUS_FS_MASK) | (SSTATUS_FS_OFF << SSTATUS_FS_SHIFT);
    write_sstatus(sstatus);
}

#[inline(always)]
unsafe fn is_fpu_enabled() -> bool {
    let sstatus = read_sstatus();
    let fs = (sstatus & SSTATUS_FS_MASK) >> SSTATUS_FS_SHIFT;
    fs != SSTATUS_FS_OFF
}

#[inline(always)]
unsafe fn is_fpu_dirty() -> bool {
    let sstatus = read_sstatus();
    let fs = (sstatus & SSTATUS_FS_MASK) >> SSTATUS_FS_SHIFT;
    fs == SSTATUS_FS_DIRTY
}

pub unsafe fn init_fpu_context_switching() {
    let sstatus = read_sstatus();
    let fs = (sstatus & SSTATUS_FS_MASK) >> SSTATUS_FS_SHIFT;
    
    if fs != SSTATUS_FS_OFF {
        FPU_AVAILABLE = true;
        
        let mstatus_test = sstatus | (SSTATUS_FS_INITIAL << SSTATUS_FS_SHIFT);
        write_sstatus(mstatus_test);
        
        let fcsr: u64;
        asm!("frcsr {}", out(reg) fcsr, options(nomem, nostack));
        asm!("fscsr zero", options(nomem, nostack));
        
        let test_val: u64 = 0xDEADBEEF_CAFEBABE;
        asm!("fmv.d.x ft0, {}", in(reg) test_val, options(nomem, nostack));
        let readback: u64;
        asm!("fmv.x.d {}, ft0", out(reg) readback, options(nomem, nostack));
        
        if readback == test_val {
            FPU_REGS_SIZE = FPU_SAVE_SIZE;
        } else {
            FPU_AVAILABLE = false;
        }
        
        write_sstatus(sstatus);
    }
    
    LAST_FPU_TASK = core::ptr::null_mut();
    LAZY_FPU_ENABLED.store(true, Ordering::Relaxed);
}

#[inline(always)]
unsafe fn save_fpu_state(task: *mut TaskStruct) {
    if task.is_null() || !FPU_AVAILABLE {
        return;
    }
    
    let ctx = &mut (*task).context;
    let fpu_area = ctx.fpu.bytes.as_mut_ptr();
    
    if !is_fpu_enabled() {
        enable_fpu();
    }
    
    asm!(
        "fsd f0, 0({0})",
        "fsd f1, 8({0})",
        "fsd f2, 16({0})",
        "fsd f3, 24({0})",
        "fsd f4, 32({0})",
        "fsd f5, 40({0})",
        "fsd f6, 48({0})",
        "fsd f7, 56({0})",
        "fsd f8, 64({0})",
        "fsd f9, 72({0})",
        "fsd f10, 80({0})",
        "fsd f11, 88({0})",
        "fsd f12, 96({0})",
        "fsd f13, 104({0})",
        "fsd f14, 112({0})",
        "fsd f15, 120({0})",
        "fsd f16, 128({0})",
        "fsd f17, 136({0})",
        "fsd f18, 144({0})",
        "fsd f19, 152({0})",
        "fsd f20, 160({0})",
        "fsd f21, 168({0})",
        "fsd f22, 176({0})",
        "fsd f23, 184({0})",
        "fsd f24, 192({0})",
        "fsd f25, 200({0})",
        "fsd f26, 208({0})",
        "fsd f27, 216({0})",
        "fsd f28, 224({0})",
        "fsd f29, 232({0})",
        "fsd f30, 240({0})",
        "fsd f31, 248({0})",
        in(reg) fpu_area,
        options(nomem, nostack)
    );
    
    let fcsr: u64;
    asm!("frcsr {}", out(reg) fcsr, options(nomem, nostack));
    
    let fcsr_slot = fpu_area.add(FPU_SAVE_SIZE);
    if FPU_SAVE_SIZE + 8 <= 512 {
        core::ptr::write(fcsr_slot as *mut u64, fcsr);
    }
    
    let mut sstatus = read_sstatus();
    sstatus = (sstatus & !SSTATUS_FS_MASK) | (SSTATUS_FS_CLEAN << SSTATUS_FS_SHIFT);
    write_sstatus(sstatus);
    
    ctx.fpu_valid = true;
    FPU_CONTEXT_SWITCHES.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
unsafe fn restore_fpu_state(task: *mut TaskStruct) {
    if task.is_null() || !FPU_AVAILABLE {
        return;
    }
    
    let ctx = &(*task).context;
    
    if !ctx.fpu_valid {
        return;
    }
    
    enable_fpu();
    
    let fpu_area = ctx.fpu.bytes.as_ptr();
    
    asm!(
        "fld f0, 0({0})",
        "fld f1, 8({0})",
        "fld f2, 16({0})",
        "fld f3, 24({0})",
        "fld f4, 32({0})",
        "fld f5, 40({0})",
        "fld f6, 48({0})",
        "fld f7, 56({0})",
        "fld f8, 64({0})",
        "fld f9, 72({0})",
        "fld f10, 80({0})",
        "fld f11, 88({0})",
        "fld f12, 96({0})",
        "fld f13, 104({0})",
        "fld f14, 112({0})",
        "fld f15, 120({0})",
        "fld f16, 128({0})",
        "fld f17, 136({0})",
        "fld f18, 144({0})",
        "fld f19, 152({0})",
        "fld f20, 160({0})",
        "fld f21, 168({0})",
        "fld f22, 176({0})",
        "fld f23, 184({0})",
        "fld f24, 192({0})",
        "fld f25, 200({0})",
        "fld f26, 208({0})",
        "fld f27, 216({0})",
        "fld f28, 224({0})",
        "fld f29, 232({0})",
        "fld f30, 240({0})",
        "fld f31, 248({0})",
        in(reg) fpu_area,
        options(nomem, nostack, readonly)
    );
    
    let fcsr_slot = fpu_area.add(FPU_SAVE_SIZE);
    if FPU_SAVE_SIZE + 8 <= 512 {
        let fcsr = core::ptr::read(fcsr_slot as *const u64);
        asm!("fscsr {}", in(reg) fcsr, options(nomem, nostack));
    }
}

#[inline(always)]
unsafe fn switch_fpu_lazy(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next {
        return;
    }
    
    if LAZY_FPU_ENABLED.load(Ordering::Relaxed) && FPU_AVAILABLE {
        if LAST_FPU_TASK == next {
            enable_fpu();
            return;
        }
        
        if !prev.is_null() && LAST_FPU_TASK == prev && is_fpu_dirty() {
            save_fpu_state(prev);
        }
        
        restore_fpu_state(next);
        LAST_FPU_TASK = next;
    } else {
        if !prev.is_null() && FPU_AVAILABLE {
            save_fpu_state(prev);
        }
        if FPU_AVAILABLE {
            restore_fpu_state(next);
        }
    }
}

#[inline(always)]
unsafe fn save_debug_registers(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let tdata1_0: u64;
    let tdata2_0: u64;
    asm!("csrr {}, 0x7a1", out(reg) tdata1_0, options(nomem, nostack, preserves_flags));
    asm!("csrr {}, 0x7a2", out(reg) tdata2_0, options(nomem, nostack, preserves_flags));
    
    let tselect_0: u64 = 0;
    asm!("csrw 0x7a0, {}", in(reg) tselect_0, options(nomem, nostack, preserves_flags));
    
    (*task).debug_regs[0] = tdata1_0;
    (*task).debug_regs[1] = tdata2_0;
}

#[inline(always)]
unsafe fn restore_debug_registers(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let tselect_0: u64 = 0;
    asm!("csrw 0x7a0, {}", in(reg) tselect_0, options(nomem, nostack, preserves_flags));
    
    let tdata1_0 = (*task).debug_regs[0];
    let tdata2_0 = (*task).debug_regs[1];
    
    asm!("csrw 0x7a1, {}", in(reg) tdata1_0, options(nomem, nostack, preserves_flags));
    asm!("csrw 0x7a2, {}", in(reg) tdata2_0, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn switch_mm(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next || next.is_null() {
        return;
    }
    
    let next_satp = (*next).mm_cr3;
    
    if next_satp == 0 {
        return;
    }
    
    let current_satp = read_satp();
    
    if current_satp != next_satp {
        let mode = (next_satp >> SATP_MODE_SHIFT) & 0xF;
        let asid = (next_satp >> 44) & ((1 << SATP_ASID_BITS) - 1);
        
        write_satp(next_satp);
        
        if asid != 0 && mode != SATP_MODE_BARE {
            sfence_vma(asid, 0);
        } else {
            sfence_vma(0, 0);
        }
    }
}

#[inline(always)]
unsafe fn switch_tp(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next || next.is_null() {
        return;
    }
    
    let next_tp = next as u64;
    write_tp(next_tp);
}

#[inline(always)]
unsafe fn switch_gp(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next || next.is_null() {
        return;
    }
    
    let next_gp = (*next).gp_base;
    if next_gp != 0 {
        write_gp(next_gp);
    }
}

#[inline(always)]
unsafe fn save_general_registers(ctx: *mut CpuContext) {
    asm!(
        "sd ra, 64({0})",
        "sd sp, 0({0})",
        "sd s0, 16({0})",
        "sd s1, 24({0})",
        "sd s2, 32({0})",
        "sd s3, 40({0})",
        "sd s4, 48({0})",
        "sd s5, 56({0})",
        in(reg) ctx,
        out("ra") _, out("sp") _, out("s0") _, out("s1") _,
        out("s2") _, out("s3") _, out("s4") _, out("s5") _,
        options(nomem, nostack)
    );
    
    let ra: u64;
    asm!("mv {}, ra", out(reg) ra, options(nomem, nostack, preserves_flags));
    (*ctx).rip = ra;
    
    let tp_val = read_tp();
    (*ctx).fs_base = tp_val;
    
    let gp_val = read_gp();
    (*ctx).gs_base = gp_val;
}

#[inline(always)]
unsafe fn restore_general_registers(ctx: *const CpuContext) {
    let ra = (*ctx).rip;
    asm!("mv ra, {}", in(reg) ra, options(nomem, nostack, preserves_flags));
    
    let tp_val = (*ctx).fs_base;
    write_tp(tp_val);
    
    let gp_val = (*ctx).gs_base;
    if gp_val != 0 {
        write_gp(gp_val);
    }
    
    asm!(
        "ld ra, 64({0})",
        "ld sp, 0({0})",
        "ld s0, 16({0})",
        "ld s1, 24({0})",
        "ld s2, 32({0})",
        "ld s3, 40({0})",
        "ld s4, 48({0})",
        "ld s5, 56({0})",
        in(reg) ctx,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn save_context_full(ctx: *mut CpuContext) {
    save_general_registers(ctx);
    
    let sstatus = read_sstatus();
    (*ctx).rflags = sstatus;
    
    let sscratch = read_sscratch();
    (*ctx).rdi = sscratch;
}

#[inline(always)]
unsafe fn restore_context_full(ctx: *const CpuContext) {
    let sstatus = (*ctx).rflags;
    write_sstatus(sstatus);
    
    let sscratch = (*ctx).rdi;
    write_sscratch(sscratch);
    
    restore_general_registers(ctx);
}

#[inline(always)]
unsafe fn validate_task_for_switch(task: *mut TaskStruct) -> bool {
    if task.is_null() {
        return false;
    }
    
    if (*task).stack_base.is_null() {
        return false;
    }
    
    if (*task).stack_size == 0 {
        return false;
    }
    
    let state = (*task).state();
    if state.is_terminal() {
        return false;
    }
    
    true
}

#[inline(always)]
unsafe fn prepare_for_switch(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if !prev.is_null() {
        (*prev).on_cpu.store(false, Ordering::Release);
    }
    
    if !next.is_null() {
        (*next).on_cpu.store(true, Ordering::Release);
        (*next).stats.nr_involuntary_switches += 1;
    }
    
    asm!("fence rw, rw", options(nomem, nostack));
}

#[inline(always)]
unsafe fn finish_switch(prev: *mut TaskStruct, next: *mut TaskStruct) {
    asm!("fence rw, rw", options(nomem, nostack));
    
    SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);
    stats::account_switch();
    
    if !next.is_null() {
        let cpu = (*next).cpu.load(Ordering::Relaxed);
        (*next).se.last_cpu = cpu;
    }
}

#[inline(never)]
pub unsafe fn switch_to(next_task: *mut TaskStruct) {
    let current_task = get_current_task();
    
    if !validate_task_for_switch(next_task) {
        panic!("Invalid task for context switch");
    }
    
    if current_task == next_task {
        return;
    }
    
    prepare_for_switch(current_task, next_task);
    
    if !current_task.is_null() {
        save_context_full(&mut (*current_task).context);
        save_debug_registers(current_task);
    }
    
    switch_mm(current_task, next_task);
    switch_tp(current_task, next_task);
    switch_gp(current_task, next_task);
    
    switch_fpu_lazy(current_task, next_task);
    
    restore_debug_registers(next_task);
    
    set_current_task(next_task);
    
    load_context(&(*next_task).context);
    
    restore_context_full(&(*next_task).context);
    
    finish_switch(current_task, next_task);
}

#[inline(always)]
unsafe fn load_context(ctx: *const CpuContext) {
    let sp = (*ctx).rsp;
    let ra = (*ctx).rip;
    
    asm!(
        "mv sp, {0}",
        "jr {1}",
        in(reg) sp,
        in(reg) ra,
        options(noreturn)
    );
}

#[inline(always)]
pub unsafe fn get_current_task() -> *mut TaskStruct {
    let current: *mut TaskStruct;
    asm!(
        "mv {}, tp",
        out(reg) current,
        options(nomem, nostack, preserves_flags)
    );
    current
}

#[inline(always)]
pub unsafe fn set_current_task(task: *mut TaskStruct) {
    asm!(
        "mv tp, {}",
        in(reg) task,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn context_switch_init() {
    init_fpu_context_switching();
    
    let sstatus = read_sstatus();
    let sstatus_new = sstatus | SSTATUS_SIE;
    write_sstatus(sstatus_new);
}

pub fn get_switch_count() -> u64 {
    SWITCH_COUNT.load(Ordering::Relaxed)
}

pub fn get_fpu_switch_count() -> u64 {
    FPU_CONTEXT_SWITCHES.load(Ordering::Relaxed)
}

pub fn enable_lazy_fpu() {
    LAZY_FPU_ENABLED.store(true, Ordering::Relaxed);
}

pub fn disable_lazy_fpu() {
    LAZY_FPU_ENABLED.store(false, Ordering::Relaxed);
}

pub fn is_lazy_fpu_enabled() -> bool {
    LAZY_FPU_ENABLED.load(Ordering::Relaxed)
}

pub unsafe fn kernel_fpu_begin() {
    if !FPU_AVAILABLE {
        return;
    }
    
    enable_fpu();
    
    if !LAST_FPU_TASK.is_null() {
        save_fpu_state(LAST_FPU_TASK);
        LAST_FPU_TASK = core::ptr::null_mut();
    }
}

pub unsafe fn kernel_fpu_end() {
    if !FPU_AVAILABLE {
        return;
    }
    
    disable_fpu();
}

pub unsafe fn flush_tlb_all() {
    sfence_vma(0, 0);
}

pub unsafe fn flush_tlb_page(vaddr: u64) {
    sfence_vma(0, vaddr);
}

pub unsafe fn flush_tlb_asid(asid: u64) {
    sfence_vma(asid, 0);
}

pub fn satp_mode_name(satp: u64) -> &'static str {
    let mode = (satp >> SATP_MODE_SHIFT) & 0xF;
    match mode {
        SATP_MODE_BARE => "Bare",
        SATP_MODE_SV39 => "Sv39",
        SATP_MODE_SV48 => "Sv48",
        SATP_MODE_SV57 => "Sv57",
        _ => "Unknown",
    }
}

pub fn satp_asid(satp: u64) -> u64 {
    (satp >> 44) & ((1 << SATP_ASID_BITS) - 1)
}

pub fn satp_ppn(satp: u64) -> u64 {
    satp & SATP_PPN_MASK
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fpu_save_area_size() {
        assert_eq!(FPU_SAVE_SIZE, 256);
        assert!(FPU_SAVE_SIZE + 8 <= 512);
    }
    
    #[test]
    fn test_satp_constants() {
        assert_eq!(SATP_MODE_SHIFT, 60);
        assert_eq!(SATP_ASID_BITS, 16);
        assert_eq!(SATP_MODE_SV39, 8);
        assert_eq!(SATP_MODE_SV48, 9);
    }
    
    #[test]
    fn test_sstatus_fs_mask() {
        assert_eq!(SSTATUS_FS_MASK, 0x3 << 13);
        assert_eq!(SSTATUS_FS_OFF, 0);
        assert_eq!(SSTATUS_FS_INITIAL, 1);
        assert_eq!(SSTATUS_FS_CLEAN, 2);
        assert_eq!(SSTATUS_FS_DIRTY, 3);
    }
    
    #[test]
    fn test_satp_mode_name() {
        assert_eq!(satp_mode_name(SATP_MODE_BARE << SATP_MODE_SHIFT), "Bare");
        assert_eq!(satp_mode_name(SATP_MODE_SV39 << SATP_MODE_SHIFT), "Sv39");
        assert_eq!(satp_mode_name(SATP_MODE_SV48 << SATP_MODE_SHIFT), "Sv48");
        assert_eq!(satp_mode_name(SATP_MODE_SV57 << SATP_MODE_SHIFT), "Sv57");
    }
}