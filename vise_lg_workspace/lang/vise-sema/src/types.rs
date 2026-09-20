#![no_std]

extern crate alloc;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymKind {
    Variable,
    Function,
    Enum,
    Static,
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClType {
    pub bits: u32,
    pub neg: bool,
}

impl ClType {
    pub const VOID: Self = Self { bits: 0, neg: false };
    pub const BOOL: Self = Self { bits: 1, neg: false };
    pub const INT: Self = Self { bits: 32, neg: true };
    pub const UINT: Self = Self { bits: 32, neg: false };
    pub const FLOAT: Self = Self { bits: 32, neg: false };
    pub const LONG: Self = Self { bits: 64, neg: true };
    pub const ULONG: Self = Self { bits: 64, neg: false };
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymKind,
    pub ty: ClType,
    pub line: u32,
    pub live: bool,
    pub freed: bool,
    pub depth: u32,
    pub pool_ref: bool,
}

impl Symbol {
    pub fn new(name: String, kind: SymKind, ty: ClType, line: u32, depth: u32) -> Self {
        Self {
            name,
            kind,
            ty,
            line,
            live: true,
            freed: false,
            depth,
            pool_ref: false,
        }
    }
}