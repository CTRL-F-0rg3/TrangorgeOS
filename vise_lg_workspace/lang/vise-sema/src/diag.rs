#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub msg: String,
    pub name: String,
    pub is_warn: bool,
}

pub struct DiagCollector {
    diags: Vec<Diagnostic>,
}

impl DiagCollector {
    pub fn new() -> Self {
        Self { diags: Vec::new() }
    }

    pub fn error(&mut self, line: u32, msg: &str, name: &str) {
        self.diags.push(Diagnostic {
            line,
            msg: String::from(msg),
            name: String::from(name),
            is_warn: false,
        });
    }

    pub fn warn(&mut self, line: u32, msg: &str, name: &str) {
        self.diags.push(Diagnostic {
            line,
            msg: String::from(msg),
            name: String::from(name),
            is_warn: true,
        });
    }

    pub fn count(&self) -> usize {
        self.diags.len()
    }

    pub fn get(&self, idx: usize) -> Option<&Diagnostic> {
        self.diags.get(idx)
    }

    pub fn has_errors(&self) -> bool {
        self.diags.iter().any(|d| !d.is_warn)
    }
}