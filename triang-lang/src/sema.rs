use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Reg(Type),
    Mem(Type, u64),
}

#[derive(Debug)]
pub struct SemaError {
    pub msg: String,
}

pub struct Sema {
    symbols: HashMap<String, Symbol>,
}

impl Sema {
    pub fn new() -> Self {
        Self { symbols: HashMap::new() }
    }

    fn err(&self, msg: String) -> SemaError {
        SemaError { msg }
    }

    pub fn check(&mut self, program: &Program) -> Result<(), SemaError> {
        for f in &program.functions {
            self.check_function(f)?;
        }
        Ok(())
    }

    fn check_function(&mut self, f: &Function) -> Result<(), SemaError> {
        self.symbols.clear();
        for s in &f.body {
            self.check_stmt(s)?;
        }
        Ok(())
    }

    fn declare(&mut self, name: &str, sym: Symbol) -> Result<(), SemaError> {
        if self.symbols.contains_key(name) {
            return Err(self.err(format!("symbol '{}' juz zadeklarowany", name)));
        }
        self.symbols.insert(name.to_string(), sym);
        Ok(())
    }

    fn check_stmt(&mut self, s: &Stmt) -> Result<(), SemaError> {
        match s {
            Stmt::RegDecl { ty, name } => {
                self.declare(name, Symbol::Reg(*ty))?;
            }
            Stmt::MemDecl { ty, len, name } => {
                self.declare(name, Symbol::Mem(*ty, *len))?;
            }
            Stmt::Op(call) => self.check_op(call)?,
            Stmt::If { cond, then_body, else_body } => {
                self.check_cond(cond)?;
                for s in then_body {
                    self.check_stmt(s)?;
                }
                for s in else_body {
                    self.check_stmt(s)?;
                }
            }
            Stmt::While { cond, body } => {
                self.check_cond(cond)?;
                for s in body {
                    self.check_stmt(s)?;
                }
            }
            Stmt::Return(e) => {
                if let Some(e) = e {
                    self.check_operand(e)?;
                }
            }
            Stmt::ConstDecl { .. } | Stmt::StaticDecl { .. } => {}
        }
        Ok(())
    }

    fn check_cond(&self, cond: &Cond) -> Result<(), SemaError> {
        self.check_operand(&cond.lhs)?;
        self.check_operand(&cond.rhs)
    }

    fn check_operand(&self, e: &Expr) -> Result<(), SemaError> {
        match e {
            Expr::Int(_) => Ok(()),
            Expr::Ident(n) => {
                if self.symbols.contains_key(n) {
                    Ok(())
                } else {
                    Err(self.err(format!("nieznany symbol '{}'", n)))
                }
            }
            Expr::Indexed(n, i) => {
                Err(self.err(format!("'{}[{}]' nie moze byc operandem tej operacji", n, i)))
            }
        }
    }

    fn check_mem_access(&self, name: &str, idx: u64) -> Result<(), SemaError> {
        match self.symbols.get(name) {
            Some(Symbol::Mem(_, len)) => {
                if idx < *len {
                    Ok(())
                } else {
                    Err(self.err(format!("indeks {} poza zakresem '{}' (len {})", idx, name, len)))
                }
            }
            Some(Symbol::Reg(_)) => {
                Err(self.err(format!("'{}' to Reg, nie mozna indeksowac", name)))
            }
            None => Err(self.err(format!("nieznany symbol '{}'", name))),
        }
    }

    fn expect_arity(&self, call: &OpCall, n: usize) -> Result<(), SemaError> {
        if call.args.len() != n {
            Err(self.err(format!("::{} oczekuje {} argumentow, jest {}", call.op, n, call.args.len())))
        } else {
            Ok(())
        }
    }

    fn check_op(&self, call: &OpCall) -> Result<(), SemaError> {
        match &call.target {
            Target::Named(name) => {
                let sym = self
                    .symbols
                    .get(name)
                    .ok_or_else(|| self.err(format!("nieznany symbol '{}'", name)))?;

                match (sym, call.op.as_str()) {
                    (Symbol::Reg(_), "set") => {
                        self.expect_arity(call, 1)?;
                        self.check_operand(&call.args[0])
                    }
                    (Symbol::Reg(_), "move") => {
                        self.expect_arity(call, 1)?;
                        self.check_operand(&call.args[0])
                    }
                    (Symbol::Reg(_), "load") => {
                        self.expect_arity(call, 1)?;
                        match &call.args[0] {
                            Expr::Indexed(n, i) => self.check_mem_access(n, *i),
                            other => Err(self.err(format!("::load oczekuje dostepu do Mem, otrzymano {:?}", other))),
                        }
                    }
                    (Symbol::Reg(_), "add" | "sub" | "mul" | "div" | "and" | "or" | "xor") => {
                        self.expect_arity(call, 2)?;
                        self.check_operand(&call.args[0])?;
                        self.check_operand(&call.args[1])
                    }
                    (Symbol::Mem(..), "set") => {
                        self.expect_arity(call, 1)?;
                        self.check_operand(&call.args[0])
                    }
                    (Symbol::Reg(..), op) => {
                        Err(self.err(format!("operacja '::{}' niedozwolona na Reg", op)))
                    }
                    (Symbol::Mem(..), op) => {
                        Err(self.err(format!("operacja '::{}' niedozwolona na Mem", op)))
                    }
                }
            }
            Target::Indexed(name, idx) => {
                self.check_mem_access(name, *idx)?;
                if call.op != "set" {
                    return Err(self.err(format!("operacja '::{}' niedozwolona na elemencie Mem", call.op)));
                }
                self.expect_arity(call, 1)?;
                self.check_operand(&call.args[0])
            }
            Target::Region => Ok(()),
        }
    }
}