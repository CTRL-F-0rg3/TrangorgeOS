#![allow(dead_code)]
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::ptr;
use crate::cpu::scheduler::entities::task::{CpuMask, SpinLock, MAX_CPUS, CPU_NONE};
use crate::cpu::scheduler::cpumask;

pub const SD_LOAD_BALANCE: u32      = 0x0001;
pub const SD_BALANCE_NEWIDLE: u32   = 0x0002;
pub const SD_BALANCE_EXEC: u32      = 0x0004;
pub const SD_BALANCE_FORK: u32      = 0x0008;
pub const SD_BALANCE_WAKE: u32      = 0x0010;
pub const SD_WAKE_AFFINE: u32       = 0x0020;
pub const SD_SHARE_CPUCAPACITY: u32 = 0x0040; // SMT (Hyper-threading)
pub const SD_SHARE_PKG_RESOURCES: u32= 0x0080; // Shared L2/L3 cache
pub const SD_SERIALIZE: u32         = 0x0100;
pub const SD_ASYM_PACKING: u32      = 0x0200;
pub const SD_NUMA: u32              = 0x0400;

pub const MAX_SCHED_DOMAIN_LEVELS: usize = 8;
pub const MAX_NUMA_NODES: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLevel {
    L1Data = 1,
    L1Instruction = 2,
    L2Unified = 3,
    L3Unified = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CacheTopology {
    pub level: CacheLevel,
    pub size_kb: u32,
    pub line_size: u32,
    pub shared_cpus: CpuMask,
}

#[repr(C)]
pub struct NumaNode {
    pub node_id: u32,
    pub cpus: CpuMask,
    pub memory_start: u64,
    pub memory_size: u64,
    pub distance: [u8; MAX_NUMA_NODES],
}

impl NumaNode {
    pub const fn empty() -> Self {
        Self {
            node_id: u32::MAX,
            cpus: CpuMask::empty(),
            memory_start: 0,
            memory_size: 0,
            distance: [0; MAX_NUMA_NODES],
        }
    }
    
    pub fn distance_to(&self, other_node: u32) -> u8 {
        if other_node as usize >= MAX_NUMA_NODES { return 255; }
        self.distance[other_node as usize]
    }
}

#[repr(C)]
pub struct SchedGroup {
    pub group_mask: CpuMask,
    pub group_weight: u32,
    pub group_capacity: u32,
    pub group_util: u32,
    pub group_misfit_task: bool,
    pub group_overloaded: bool,
    pub next: *mut SchedGroup,
    pub idle_cpus: CpuMask,
    pub rq_avg_load: u64,
}

impl SchedGroup {
    pub const fn empty() -> Self {
        Self {
            group_mask: CpuMask::empty(),
            group_weight: 0,
            group_capacity: 0,
            group_util: 0,
            group_misfit_task: false,
            group_overloaded: false,
            next: ptr::null_mut(),
            idle_cpus: CpuMask::empty(),
            rq_avg_load: 0,
        }
    }
}

#[repr(C)]
pub struct SchedDomain {
    pub level: u32,
    pub flags: u32,
    pub span: CpuMask,
    pub child: *mut SchedDomain,
    pub parent: *mut SchedDomain,
    pub groups: *mut SchedGroup,
    pub nr_groups: u32,
    pub cache_nice_tries: u32,
    pub busy_idx: u32,
    pub idle_idx: u32,
    pub newidle_idx: u32,
    pub wake_idx: u32,
    pub fork_idx: u32,
    pub last_balance: AtomicU64,
    pub nr_balance_failed: AtomicU32,
    pub domain_name: &'static str,
}

impl SchedDomain {
    pub const fn empty() -> Self {
        Self {
            level: 0,
            flags: 0,
            span: CpuMask::empty(),
            child: ptr::null_mut(),
            parent: ptr::null_mut(),
            groups: ptr::null_mut(),
            nr_groups: 0,
            cache_nice_tries: 2,
            busy_idx: 0,
            idle_idx: 0,
            newidle_idx: 0,
            wake_idx: 0,
            fork_idx: 0,
            last_balance: AtomicU64::new(0),
            nr_balance_failed: AtomicU32::new(0),
            domain_name: "unknown",
        }
    }

    pub fn has_flag(&self, flag: u32) -> bool {
        (self.flags & flag) != 0
    }

    pub fn is_numa(&self) -> bool {
        self.has_flag(SD_NUMA)
    }

    pub fn shares_cache(&self) -> bool {
        self.has_flag(SD_SHARE_PKG_RESOURCES)
    }

    pub fn is_smt(&self) -> bool {
        self.has_flag(SD_SHARE_CPUCAPACITY)
    }
}

#[repr(C)]
pub struct CpuTopology {
    pub cpu_id: u32,
    pub package_id: u32,
    pub core_id: u32,
    pub smt_id: u32,
    pub numa_node_id: u32,
    pub l1d_cache: CacheTopology,
    pub l2_cache: CacheTopology,
    pub l3_cache: CacheTopology,
    pub thread_siblings: CpuMask,
    pub core_siblings: CpuMask,
    pub book_siblings: CpuMask,
    pub drawer_siblings: CpuMask,
}

impl CpuTopology {
    pub const fn empty() -> Self {
        const EMPTY_CACHE: CacheTopology = CacheTopology {
            level: CacheLevel::L1Data, size_kb: 0, line_size: 0, shared_cpus: CpuMask::empty()
        };
        Self {
            cpu_id: CPU_NONE,
            package_id: u32::MAX,
            core_id: u32::MAX,
            smt_id: u32::MAX,
            numa_node_id: 0,
            l1d_cache: EMPTY_CACHE,
            l2_cache: EMPTY_CACHE,
            l3_cache: EMPTY_CACHE,
            thread_siblings: CpuMask::empty(),
            core_siblings: CpuMask::empty(),
            book_siblings: CpuMask::empty(),
            drawer_siblings: CpuMask::empty(),
        }
    }
}

pub struct TopologyState {
    pub cpus: [CpuTopology; MAX_CPUS],
    pub nodes: [NumaNode; MAX_NUMA_NODES],
    pub nr_nodes: u32,
    pub sd_pool: [SchedDomain; MAX_CPUS * MAX_SCHED_DOMAIN_LEVELS],
    pub sg_pool: [SchedGroup; MAX_CPUS * MAX_SCHED_DOMAIN_LEVELS],
    pub sd_used: AtomicU32,
    pub sg_used: AtomicU32,
    pub per_cpu_domains: [*mut SchedDomain; MAX_CPUS],
    pub lock: SpinLock,
    pub initialized: AtomicBool,
}

impl TopologyState {
    pub const fn empty() -> Self {
        const EMPTY_CPU: CpuTopology = CpuTopology::empty();
        const EMPTY_NODE: NumaNode = NumaNode::empty();
        const EMPTY_SD: SchedDomain = SchedDomain::empty();
        const EMPTY_SG: SchedGroup = SchedGroup::empty();
        
        Self {
            cpus: [EMPTY_CPU; MAX_CPUS],
            nodes: [EMPTY_NODE; MAX_NUMA_NODES],
            nr_nodes: 0,
            sd_pool: [EMPTY_SD; MAX_CPUS * MAX_SCHED_DOMAIN_LEVELS],
            sg_pool: [EMPTY_SG; MAX_CPUS * MAX_SCHED_DOMAIN_LEVELS],
            sd_used: AtomicU32::new(0),
            sg_used: AtomicU32::new(0),
            per_cpu_domains: [ptr::null_mut(); MAX_CPUS],
            lock: SpinLock::new(),
            initialized: AtomicBool::new(false),
        }
    }

    pub fn alloc_domain(&mut self) -> Option<&mut SchedDomain> {
        let idx = self.sd_used.load(Ordering::Relaxed) as usize;
        if idx >= self.sd_pool.len() { return None; }
        self.sd_used.fetch_add(1, Ordering::Relaxed);
        self.sd_pool[idx] = SchedDomain::empty();
        Some(&mut self.sd_pool[idx])
    }

    pub fn alloc_group(&mut self) -> Option<&mut SchedGroup> {
        let idx = self.sg_used.load(Ordering::Relaxed) as usize;
        if idx >= self.sg_pool.len() { return None; }
        self.sg_used.fetch_add(1, Ordering::Relaxed);
        self.sg_pool[idx] = SchedGroup::empty();
        Some(&mut self.sg_pool[idx])
    }
}

pub static mut GLOBAL_TOPOLOGY: TopologyState = TopologyState::empty();

pub unsafe fn topology_init() {
    let _guard = GLOBAL_TOPOLOGY.lock.lock_irqsave_guard();
    if GLOBAL_TOPOLOGY.initialized.load(Ordering::Relaxed) { return; }
    
    // W środowisku produkcyjnym tutaj parsujesz ACPI SRAT/SLIT lub Device Tree.
    // Na potrzeby stabilizacji budujemy syntetyczną topologię.
    build_synthetic_topology();
    
    GLOBAL_TOPOLOGY.initialized.store(true, Ordering::Release);
}

unsafe fn build_synthetic_topology() {
    let online = cpumask::online_mask();
    if online.is_empty() { return; }

    let mut node_idx = 0;
    GLOBAL_TOPOLOGY.nodes[node_idx].node_id = 0;
    GLOBAL_TOPOLOGY.nodes[node_idx].distance[0] = 10;
    GLOBAL_TOPOLOGY.nr_nodes = 1;

    for cpu in online.iter() {
        let cpu_usize = cpu as usize;
        if cpu_usize >= MAX_CPUS { continue; }
        
        let topo = &mut GLOBAL_TOPOLOGY.cpus[cpu_usize];
        topo.cpu_id = cpu;
        topo.package_id = cpu / 8; 
        topo.core_id = cpu / 2;    
        topo.smt_id = cpu % 2;
        topo.numa_node_id = 0;
        
        GLOBAL_TOPOLOGY.nodes[0].cpus.set(cpu);
        
        topo.thread_siblings = cpumask::sibling_mask(cpu);
        topo.core_siblings = cpumask::package_mask(cpu);
        
        topo.l1d_cache.level = CacheLevel::L1Data;
        topo.l1d_cache.size_kb = 48;
        topo.l1d_cache.line_size = 64;
        topo.l1d_cache.shared_cpus = CpuMask::single(cpu);
        
        topo.l2_cache.level = CacheLevel::L2Unified;
        topo.l2_cache.size_kb = 1024;
        topo.l2_cache.line_size = 64;
        topo.l2_cache.shared_cpus = topo.thread_siblings;
        
        topo.l3_cache.level = CacheLevel::L3Unified;
        topo.l3_cache.size_kb = 16384;
        topo.l3_cache.line_size = 64;
        topo.l3_cache.shared_cpus = topo.core_siblings;
    }

    build_sched_domains();
}

unsafe fn build_sched_domains() {
    let online = cpumask::online_mask();
    
    for cpu in online.iter() {
        let cpu_usize = cpu as usize;
        if cpu_usize >= MAX_CPUS { continue; }

        let mut prev_sd: *mut SchedDomain = ptr::null_mut();
        
        // Level 0: SMT (Thread siblings)
        let smt_mask = GLOBAL_TOPOLOGY.cpus[cpu_usize].thread_siblings;
        if smt_mask.count() > 1 {
            if let Some(sd) = GLOBAL_TOPOLOGY.alloc_domain() {
                sd.level = 0;
                sd.flags = SD_LOAD_BALANCE | SD_BALANCE_NEWIDLE | SD_BALANCE_EXEC | SD_BALANCE_WAKE | SD_SHARE_CPUCAPACITY;
                sd.span = smt_mask;
                sd.domain_name = "SMT";
                sd.cache_nice_tries = 1;
                
                let sd_ptr = sd as *mut SchedDomain;
                if !prev_sd.is_null() { (*prev_sd).parent = sd_ptr; }
                prev_sd = sd_ptr;
            }
        }

        // Level 1: MC (Multi-Core / L3 Cache)
        let mc_mask = GLOBAL_TOPOLOGY.cpus[cpu_usize].core_siblings;
        if mc_mask.count() > 1 && mc_mask != smt_mask {
            if let Some(sd) = GLOBAL_TOPOLOGY.alloc_domain() {
                sd.level = 1;
                sd.flags = SD_LOAD_BALANCE | SD_BALANCE_NEWIDLE | SD_BALANCE_EXEC | SD_BALANCE_WAKE | SD_SHARE_PKG_RESOURCES;
                sd.span = mc_mask;
                sd.domain_name = "MC";
                sd.cache_nice_tries = 2;
                
                let sd_ptr = sd as *mut SchedDomain;
                if !prev_sd.is_null() { 
                    (*prev_sd).parent = sd_ptr; 
                    (*sd_ptr).child = prev_sd;
                }
                prev_sd = sd_ptr;
            }
        }

        // Level 2: NUMA / PKG (Package)
        let pkg_mask = GLOBAL_TOPOLOGY.nodes[0].cpus; 
        if pkg_mask.count() > 1 && pkg_mask != mc_mask {
            if let Some(sd) = GLOBAL_TOPOLOGY.alloc_domain() {
                sd.level = 2;
                sd.flags = SD_LOAD_BALANCE | SD_BALANCE_NEWIDLE | SD_NUMA | SD_ASYM_PACKING;
                sd.span = pkg_mask;
                sd.domain_name = "NUMA";
                sd.cache_nice_tries = 3;
                
                let sd_ptr = sd as *mut SchedDomain;
                if !prev_sd.is_null() { 
                    (*prev_sd).parent = sd_ptr; 
                    (*sd_ptr).child = prev_sd;
                }
                prev_sd = sd_ptr;
            }
        }

        GLOBAL_TOPOLOGY.per_cpu_domains[cpu_usize] = prev_sd;
    }
}

pub unsafe fn cpu_topology(cpu: u32) -> &'static CpuTopology {
    if cpu as usize >= MAX_CPUS { panic!("Invalid CPU ID"); }
    &GLOBAL_TOPOLOGY.cpus[cpu as usize]
}

pub unsafe fn lowest_shared_cache_level(cpu_a: u32, cpu_b: u32) -> Option<CacheLevel> {
    let topo_a = cpu_topology(cpu_a);
    let topo_b = cpu_topology(cpu_b);

    if topo_a.l1d_cache.shared_cpus.is_set(cpu_b) { return Some(CacheLevel::L1Data); }
    if topo_a.l2_cache.shared_cpus.is_set(cpu_b) { return Some(CacheLevel::L2Unified); }
    if topo_a.l3_cache.shared_cpus.is_set(cpu_b) { return Some(CacheLevel::L3Unified); }
    
    None
}

pub unsafe fn numa_distance(node_a: u32, node_b: u32) -> u8 {
    if node_a as usize >= MAX_NUMA_NODES || node_b as usize >= MAX_NUMA_NODES { return 255; }
    GLOBAL_TOPOLOGY.nodes[node_a as usize].distance_to(node_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_alloc_domains_respects_limits() {
        unsafe {
            let mut state = TopologyState::empty();
            let mut count = 0;
            while state.alloc_domain().is_some() {
                count += 1;
            }
            assert_eq!(count, MAX_CPUS * MAX_SCHED_DOMAIN_LEVELS);
            assert!(state.alloc_domain().is_none());
        }
    }

    #[test]
    fn sched_domain_flags_logic() {
        let mut sd = SchedDomain::empty();
        sd.flags = SD_LOAD_BALANCE | SD_SHARE_CPUCAPACITY;
        assert!(sd.has_flag(SD_LOAD_BALANCE));
        assert!(sd.is_smt());
        assert!(!sd.is_numa());
        assert!(!sd.shares_cache());
    }
}