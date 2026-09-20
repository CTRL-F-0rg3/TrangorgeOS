#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use super::types::Symbol;

pub struct Scope {
    pub symbols: Vec<Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self { symbols: Vec::new() }
    }

    pub fn declare(&mut self, sym: Symbol) {
        self.symbols.push(sym);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn current_depth(&self) -> u32 {
        self.scopes.len() as u32
    }

    pub fn declare(&mut self, sym: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.declare(sym);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.lookup(name) {
                return Some(sym);
            }
        }
        None
    }
}