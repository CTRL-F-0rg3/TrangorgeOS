#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    X86_64,
    Aarch64,
    RiscV64,
}

impl Target {
    pub fn reg(&self, name: &str) -> &'static str {
        let idx: usize = name.trim_start_matches('r').parse().unwrap_or(0);
        let tbl: &[&str] = match self {
            Target::X86_64 => &["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"],
            Target::Aarch64 => &["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9"],
            Target::RiscV64 => &["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "t0", "t1"],
        };
        tbl[idx.min(tbl.len() - 1)]
    }

    pub fn ret_reg(&self) -> &'static str {
        match self {
            Target::X86_64 => "rax",
            Target::Aarch64 => "x0",
            Target::RiscV64 => "a0",
        }
    }
}