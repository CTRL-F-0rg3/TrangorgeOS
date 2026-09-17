#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::cpu::scheduler::entities::task::{
    TaskStruct, TaskFlags, CpuContext, FxSaveArea, KERNEL_STACK_SIZE,
};
use crate::cpu::scheduler::stats;

const XSAVE_AREA_SIZE: usize = 832;
const FXSAVE_AREA_SIZE: usize = 512;
const XCR0_SSE: u64 = 1 << 1;
const XCR0_AVX: u64 = 1 << 2;
const XCR0_MPX: u64 = (1 << 3) | (1 << 4);
const XCR0_AVX512: u64 = (1 << 5) | (1 << 6) | (1 << 7);

const CR3_PCID_MASK: u64 = 0xFFF;
const CR3_NO_FLUSH: u64 = 1 << 63;

const MSR_FS_BASE: u32 = 0xC0000100;
const MSR_GS_BASE: u32 = 0xC0000101;
const MSR_KERNEL_GS_BASE: u32 = 0xC0000102;
const MSR_STAR: u32 = 0xC0000081;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_CSTAR: u32 = 0xC0000083;
const MSR_SFMASK: u32 = 0xC0000084;
const MSR_EFER: u32 = 0xC0000080;
const MSR_TSC_AUX: u32 = 0xC0000103;

const DEBUG_REGISTERS_COUNT: usize = 8;

static SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
static FPU_CONTEXT_SWITCHES: AtomicU64 = AtomicU64::new(0);
static LAZY_FPU_ENABLED: AtomicBool = AtomicBool::new(true);

static mut LAST_FPU_TASK: *mut TaskStruct = core::ptr::null_mut();
static mut XSAVE_ENABLED: bool = false;
static mut AVX_ENABLED: bool = false;
static mut AVX512_ENABLED: bool = false;
static mut MPX_ENABLED: bool = false;
static mut XSAVE_SIZE: usize = FXSAVE_AREA_SIZE;

#[repr(C, align(64))]
struct XSaveArea {
    data: [u8; XSAVE_AREA_SIZE],
}

impl XSaveArea {
    const fn new() -> Self {
        Self { data: [0; XSAVE_AREA_SIZE] }
    }
}

#[repr(C)]
struct ExtendedContext {
    xsave_area: XSaveArea,
    xcomp_bv: u64,
    header_bv: u64,
}

#[inline(always)]
unsafe fn read_cr3() -> u64 {
    let cr3: u64;
    asm!("mov {0}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    cr3
}

#[inline(always)]
unsafe fn write_cr3(cr3: u64) {
    asm!("mov cr3, {0}", in(reg) cr3, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_cr0() -> u64 {
    let cr0: u64;
    asm!("mov {0}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    cr0
}

#[inline(always)]
unsafe fn read_cr4() -> u64 {
    let cr4: u64;
    asm!("mov {0}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
    cr4
}

#[inline(always)]
unsafe fn write_cr4(cr4: u64) {
    asm!("mov cr4, {0}", in(reg) cr4, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

#[inline(always)]
unsafe fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_xcr0() -> u64 {
    let low: u32;
    let high: u32;
    asm!("xgetbv", in("ecx") 0u32, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

#[inline(always)]
unsafe fn clflush(addr: *const u8) {
    asm!("clflush [{0}]", in(reg) addr, options(nomem, nostack));
}

#[inline(always)]
unsafe fn mfence() {
    asm!("mfence", options(nomem, nostack));
}

#[inline(always)]
unsafe fn lfence() {
    asm!("lfence", options(nomem, nostack));
}

#[inline(always)]
unsafe fn sfence() {
    asm!("sfence", options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn clts() {
    asm!("clts", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn stts() {
    let cr0 = read_cr0();
    write_cr0(cr0 | 0x8);
}

pub unsafe fn init_fpu_context_switching() {
    let cr4 = read_cr4();
    
    write_cr4(cr4 | (1 << 9));
    
    let cpuid_eax: u32;
    let cpuid_ecx: u32;
    asm!(
        "cpuid",
        in("eax") 0xDu32,
        in("ecx") 0u32,
        out("eax") cpuid_eax,
        out("ecx") cpuid_ecx,
        out("edx") _,
        out("ebx") _,
        options(nomem, nostack)
    );
    
    if cpuid_ecx & 0x1 != 0 {
        XSAVE_ENABLED = true;
        XSAVE_SIZE = cpuid_eax as usize;
        
        let xcr0 = read_xcr0();
        
        let cpuid_eax2: u32;
        let cpuid_ecx2: u32;
        asm!(
            "cpuid",
            in("eax") 7u32,
            in("ecx") 0u32,
            out("eax") _,
            out("ebx") _,
            out("ecx") cpuid_ecx2,
            out("edx") cpuid_eax2,
            options(nomem, nostack)
        );
        
        if cpuid_ecx2 & (1 << 5) != 0 {
            AVX_ENABLED = true;
            write_xcr0(xcr0 | XCR0_AVX);
        }
        
        if cpuid_eax2 & (1 << 16) != 0 {
            AVX512_ENABLED = true;
            write_xcr0(xcr0 | XCR0_AVX512);
        }
        
        if cpuid_eax2 & (1 << 14) != 0 {
            MPX_ENABLED = true;
            write_xcr0(xcr0 | XCR0_MPX);
        }
    }
    
    LAST_FPU_TASK = core::ptr::null_mut();
    LAZY_FPU_ENABLED.store(true, Ordering::Relaxed);
}

#[inline(always)]
unsafe fn write_xcr0(value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!("xsetbv", in("ecx") 0u32, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn fxsave(area: *mut u8) {
    asm!(
        "fxsave [{0}]",
        in(reg) area,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn fxrstor(area: *const u8) {
    asm!(
        "fxrstor [{0}]",
        in(reg) area,
        options(nomem, nostack, readonly)
    );
}

#[inline(always)]
unsafe fn xsave(area: *mut u8, mask: u64) {
    let low = mask as u32;
    let high = (mask >> 32) as u32;
    asm!(
        "xsave [{0}]",
        in(reg) area,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn xrstor(area: *const u8, mask: u64) {
    let low = mask as u32;
    let high = (mask >> 32) as u32;
    asm!(
        "xrstor [{0}]",
        in(reg) area,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, readonly)
    );
}

#[inline(always)]
unsafe fn xsaveopt(area: *mut u8, mask: u64) {
    let low = mask as u32;
    let high = (mask >> 32) as u32;
    asm!(
        "xsaveopt [{0}]",
        in(reg) area,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn save_fpu_state(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let ctx = &mut (*task).context;
    
    if XSAVE_ENABLED {
        let mask = if AVX512_ENABLED {
            0xFFFFFFFFFFFFFFFF
        } else if AVX_ENABLED {
            0x7
        } else {
            0x3
        };
        
        if AVX512_ENABLED || AVX_ENABLED {
            xsaveopt(ctx.fpu.bytes.as_mut_ptr(), mask);
        } else {
            xsave(ctx.fpu.bytes.as_mut_ptr(), mask);
        }
    } else {
        fxsave(ctx.fpu.bytes.as_mut_ptr());
    }
    
    ctx.fpu_valid = true;
    FPU_CONTEXT_SWITCHES.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
unsafe fn restore_fpu_state(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let ctx = &(*task).context;
    
    if !ctx.fpu_valid {
        return;
    }
    
    clts();
    
    if XSAVE_ENABLED {
        let mask = if AVX512_ENABLED {
            0xFFFFFFFFFFFFFFFF
        } else if AVX_ENABLED {
            0x7
        } else {
            0x3
        };
        
        xrstor(ctx.fpu.bytes.as_ptr(), mask);
    } else {
        fxrstor(ctx.fpu.bytes.as_ptr());
    }
}

#[inline(always)]
unsafe fn switch_fpu_lazy(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next {
        return;
    }
    
    if LAZY_FPU_ENABLED.load(Ordering::Relaxed) {
        if LAST_FPU_TASK == next {
            return;
        }
        
        if !prev.is_null() && LAST_FPU_TASK == prev {
            save_fpu_state(prev);
        }
        
        restore_fpu_state(next);
        LAST_FPU_TASK = next;
    } else {
        if !prev.is_null() {
            save_fpu_state(prev);
        }
        restore_fpu_state(next);
    }
}

#[inline(always)]
unsafe fn save_debug_registers(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let dr0: u64;
    let dr1: u64;
    let dr2: u64;
    let dr3: u64;
    let dr6: u64;
    
    asm!("mov {0}, dr0", out(reg) dr0, options(nomem, nostack, preserves_flags));
    asm!("mov {0}, dr1", out(reg) dr1, options(nomem, nostack, preserves_flags));
    asm!("mov {0}, dr2", out(reg) dr2, options(nomem, nostack, preserves_flags));
    asm!("mov {0}, dr3", out(reg) dr3, options(nomem, nostack, preserves_flags));
    asm!("mov {0}, dr6", out(reg) dr6, options(nomem, nostack, preserves_flags));
    
    (*task).debug_regs[0] = dr0;
    (*task).debug_regs[1] = dr1;
    (*task).debug_regs[2] = dr2;
    (*task).debug_regs[3] = dr3;
    (*task).debug_regs[6] = dr6;
}

#[inline(always)]
unsafe fn restore_debug_registers(task: *mut TaskStruct) {
    if task.is_null() {
        return;
    }
    
    let dr0 = (*task).debug_regs[0];
    let dr1 = (*task).debug_regs[1];
    let dr2 = (*task).debug_regs[2];
    let dr3 = (*task).debug_regs[3];
    let dr6 = (*task).debug_regs[6];
    let dr7 = (*task).debug_regs[7];
    
    asm!("mov dr0, {0}", in(reg) dr0, options(nomem, nostack, preserves_flags));
    asm!("mov dr1, {0}", in(reg) dr1, options(nomem, nostack, preserves_flags));
    asm!("mov dr2, {0}", in(reg) dr2, options(nomem, nostack, preserves_flags));
    asm!("mov dr3, {0}", in(reg) dr3, options(nomem, nostack, preserves_flags));
    asm!("mov dr6, {0}", in(reg) dr6, options(nomem, nostack, preserves_flags));
    asm!("mov dr7, {0}", in(reg) dr7, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn switch_mm(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next {
        return;
    }
    
    if next.is_null() {
        return;
    }
    
    let next_cr3 = (*next).mm_cr3;
    
    if next_cr3 == 0 {
        return;
    }
    
    let current_cr3 = read_cr3();
    
    if current_cr3 != next_cr3 {
        let pcid = next_cr3 & CR3_PCID_MASK;
        let new_cr3 = if pcid != 0 {
            next_cr3 | CR3_NO_FLUSH
        } else {
            next_cr3
        };
        
        write_cr3(new_cr3);
    }
}

#[inline(always)]
unsafe fn switch_tss(next: *mut TaskStruct) {
    if next.is_null() {
        return;
    }
    
    let kernel_rsp = (*next).kernel_rsp;
    if kernel_rsp == 0 {
        return;
    }
    
    let tss_ptr = (*next).tss_ptr;
    if tss_ptr.is_null() {
        return;
    }
    
    let tss = &mut *(tss_ptr as *mut TssStruct);
    tss.rsp0 = kernel_rsp;
    
    let tss_selector = (*next).tss_selector;
    if tss_selector != 0 {
        asm!(
            "ltr {0:x}",
            in(reg) tss_selector,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[repr(C, packed)]
struct TssStruct {
    reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved2: u64,
    reserved3: u16,
    iomap_base: u16,
}

#[inline(always)]
unsafe fn switch_fs_gs(prev: *mut TaskStruct, next: *mut TaskStruct) {
    if prev == next {
        return;
    }
    
    if !prev.is_null() {
        let fs_base: u64;
        asm!("rdmsr", in("ecx") MSR_FS_BASE, out("eax") _, out("edx") _, options(nomem, nostack, preserves_flags));
        asm!("", out("rax") fs_base, options(nomem, nostack, preserves_flags));
        (*prev).fs_base = fs_base;
        
        let gs_base: u64;
        asm!("rdmsr", in("ecx") MSR_KERNEL_GS_BASE, out("eax") _, out("edx") _, options(nomem, nostack, preserves_flags));
        asm!("", out("rax") gs_base, options(nomem, nostack, preserves_flags));
        (*prev).user_gs_base = gs_base;
    }
    
    if !next.is_null() {
        let fs_base = (*next).fs_base;
        let fs_low = fs_base as u32;
        let fs_high = (fs_base >> 32) as u32;
        asm!("wrmsr", in("ecx") MSR_FS_BASE, in("eax") fs_low, in("edx") fs_high, options(nomem, nostack, preserves_flags));
        
        let gs_base = (*next).user_gs_base;
        let gs_low = gs_base as u32;
        let gs_high = (gs_base >> 32) as u32;
        asm!("wrmsr", in("ecx") MSR_KERNEL_GS_BASE, in("eax") gs_low, in("edx") gs_high, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
unsafe fn save_general_registers(ctx: *mut CpuContext) {
    asm!(
        "mov [{0} + 0x00], rbx",
        "mov [{0} + 0x08], rbp",
        "mov [{0} + 0x10], r12",
        "mov [{0} + 0x18], r13",
        "mov [{0} + 0x20], r14",
        "mov [{0} + 0x28], r15",
        "mov [{0} + 0x30], rsp",
        in(reg) ctx,
        out("rbx") _, out("rbp") _, out("r12") _, out("r13") _,
        out("r14") _, out("r15") _,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn restore_general_registers(ctx: *const CpuContext) {
    asm!(
        "mov rbx, [{0} + 0x00]",
        "mov rbp, [{0} + 0x08]",
        "mov r12, [{0} + 0x10]",
        "mov r13, [{0} + 0x18]",
        "mov r14, [{0} + 0x20]",
        "mov r15, [{0} + 0x28]",
        "mov rsp, [{0} + 0x30]",
        in(reg) ctx,
        options(nomem, nostack)
    );
}

#[inline(always)]
unsafe fn save_segment_registers(ctx: *mut CpuContext) {
    let fs: u16;
    let gs: u16;
    
    asm!("mov {0:x}, fs", out(reg) fs, options(nomem, nostack, preserves_flags));
    asm!("mov {0:x}, gs", out(reg) gs, options(nomem, nostack, preserves_flags));
    
    (*ctx).fs_segment = fs;
    (*ctx).gs_segment = gs;
}

#[inline(always)]
unsafe fn restore_segment_registers(ctx: *const CpuContext) {
    let fs = (*ctx).fs_segment;
    let gs = (*ctx).gs_segment;
    
    asm!("mov fs, {0:x}", in(reg) fs, options(nomem, nostack, preserves_flags));
    asm!("mov gs, {0:x}", in(reg) gs, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn save_control_registers(ctx: *mut CpuContext) {
    let cr2: u64;
    asm!("mov {0}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    (*ctx).cr2 = cr2;
    
    let efer = read_msr(MSR_EFER);
    (*ctx).efer = efer;
}

#[inline(always)]
unsafe fn restore_control_registers(ctx: *const CpuContext) {
    let cr2 = (*ctx).cr2;
    asm!("mov cr2, {0}", in(reg) cr2, options(nomem, nostack, preserves_flags));
    
    let efer = (*ctx).efer;
    write_msr(MSR_EFER, efer);
}

#[inline(always)]
unsafe fn save_context_full(ctx: *mut CpuContext) {
    save_general_registers(ctx);
    save_segment_registers(ctx);
    save_control_registers(ctx);
    
    let rflags: u64;
    asm!("pushfq; pop {0}", out(reg) rflags, options(nomem));
    (*ctx).rflags = rflags;
}

#[inline(always)]
unsafe fn restore_context_full(ctx: *const CpuContext) {
    let rflags = (*ctx).rflags;
    asm!("push {0}; popfq", in(reg) rflags, options(nomem));
    
    restore_control_registers(ctx);
    restore_segment_registers(ctx);
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
    
    mfence();
}

#[inline(always)]
unsafe fn finish_switch(prev: *mut TaskStruct, next: *mut TaskStruct) {
    sfence();
    
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
    switch_tss(next_task);
    switch_fs_gs(current_task, next_task);
    
    switch_fpu_lazy(current_task, next_task);
    
    restore_debug_registers(next_task);
    
    set_current_task(next_task);
    
    load_context(&(*next_task).context);
    
    restore_context_full(&(*next_task).context);
    
    finish_switch(current_task, next_task);
}

#[inline(always)]
unsafe fn load_context(ctx: *const CpuContext) {
    let rsp = (*ctx).rsp;
    let rip = (*ctx).rip;
    
    asm!(
        "mov rsp, {0}",
        "jmp {1}",
        in(reg) rsp,
        in(reg) rip,
        options(noreturn)
    );
}

#[inline(always)]
pub unsafe fn get_current_task() -> *mut TaskStruct {
    let current: *mut TaskStruct;
    asm!(
        "mov {0}, gs:[{1}]",
        out(reg) current,
        const 0,
        options(nomem, nostack, preserves_flags)
    );
    current
}

#[inline(always)]
pub unsafe fn set_current_task(task: *mut TaskStruct) {
    asm!(
        "mov gs:[{0}], {1}",
        const 0,
        in(reg) task,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn context_switch_init() {
    init_fpu_context_switching();
    
    let cr0 = read_cr0();
    write_cr0(cr0 | 0x2);
    
    let cr4 = read_cr4();
    write_cr4(cr4 | (1 << 7));
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
    clts();
    
    if !LAST_FPU_TASK.is_null() {
        save_fpu_state(LAST_FPU_TASK);
        LAST_FPU_TASK = core::ptr::null_mut();
    }
}

pub unsafe fn kernel_fpu_end() {
    stts();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xsave_area_alignment() {
        assert_eq!(core::mem::align_of::<XSaveArea>(), 64);
    }
    
    #[test]
    fn test_msr_constants() {
        assert_eq!(MSR_FS_BASE, 0xC0000100);
        assert_eq!(MSR_GS_BASE, 0xC0000101);
        assert_eq!(MSR_KERNEL_GS_BASE, 0xC0000102);
    }
    
    #[test]
    fn test_cr3_masks() {
        assert_eq!(CR3_PCID_MASK, 0xFFF);
        assert_eq!(CR3_NO_FLUSH, 1 << 63);
    }
}