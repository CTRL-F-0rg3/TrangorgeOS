#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use vise_parser::ast::*;
use super::types::*;
use super::scope::ScopeStack;
use super::diag::DiagCollector;

pub struct Sema {
    pub scopes: ScopeStack,
    pub diags: DiagCollector,
}

impl Sema {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            diags: DiagCollector::new(),
        }
    }

    pub fn run(&mut self, module: &Module) {
        self.scopes.push();
        
        for item in &module.items {
            self.declare_top(item);
        }
        
        for item in &module.items {
            self.check_top(item);
        }
        
        self.scopes.pop();
    }

    fn declare_top(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(func) => {
                let ty = match func.return_type {
                    Type::Float => ClType::FLOAT,
                    Type::Int => ClType::INT,
                    Type::Uint => ClType::UINT,
                    _ => ClType::VOID,
                };
                
                if self.scopes.lookup(&func.name).is_some() {
                    self.diags.error(0, "redeclaration", &func.name);
                }
                
                let sym = Symbol::new(
                    func.name.clone(),
                    SymKind::Function,
                    ty,
                    0,
                    0,
                );
                self.scopes.declare(sym);
            }
            TopLevel::Struct(_) => {}
            TopLevel::Enum(_) => {}
            TopLevel::Static(_) => {}
        }
    }

    fn check_top(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(func) => {
                self.scopes.push();
                
                for param in &func.params {
                    let ty = match param.ty {
                        Type::Float => ClType::FLOAT,
                        Type::Int => ClType::INT,
                        Type::Uint => ClType::UINT,
                        _ => ClType::VOID,
                    };
                    
                    let sym = Symbol::new(
                        param.name.clone(),
                        SymKind::Variable,
                        ty,
                        0,
                        self.scopes.current_depth(),
                    );
                    self.scopes.declare(sym);
                }
                
                for stmt in &func.body {
                    self.check_stmt(stmt);
                }
                
                self.scopes.pop();
            }
            _ => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty, init, .. } => {
                let init_ty = self.check_expr(init);
                
                let cl_ty = match ty {
                    Type::Float => ClType::FLOAT,
                    Type::Int => ClType::INT,
                    Type::Uint => ClType::UINT,
                    _ => ClType::VOID,
                };
                
                if self.scopes.lookup(name).is_some() {
                    self.diags.error(0, "redeclaration in same scope", name);
                }
                
                let sym = Symbol::new(
                    name.clone(),
                    SymKind::Variable,
                    cl_ty,
                    0,
                    self.scopes.current_depth(),
                );
                self.scopes.declare(sym);
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e);
                }
            }
            Stmt::If { cond, then_block, else_block } => {
                self.check_expr(cond);
                
                self.scopes.push();
                for s in then_block {
                    self.check_stmt(s);
                }
                self.scopes.pop();
                
                if let Some(else_stmts) = else_block {
                    self.scopes.push();
                    for s in else_stmts {
                        self.check_stmt(s);
                    }
                    self.scopes.pop();
                }
            }
            _ => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> ClType {
        match expr {
            Expr::IntLit(_) => ClType::INT,
            Expr::FloatLit(_) => ClType::FLOAT,
            Expr::BoolLit(_) => ClType::BOOL,
            Expr::Ident(name) => {
                if let Some(sym) = self.scopes.lookup(name) {
                    if !sym.live {
                        self.diags.error(0, "use after set_free", name);
                    }
                    sym.ty
                } else {
                    self.diags.error(0, "undeclared identifier", name);
                    ClType::VOID
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                let lhs_ty = self.check_expr(lhs);
                let rhs_ty = self.check_expr(rhs);
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if lhs_ty.bits > rhs_ty.bits {
                            lhs_ty
                        } else {
                            rhs_ty
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        ClType::BOOL
                    }
                    _ => lhs_ty,
                }
            }
            Expr::Unary { op, operand } => {
                self.check_expr(operand)
            }
            Expr::Call { func, args } => {
                if let Some(sym) = self.scopes.lookup(func) {
                    if sym.kind != SymKind::Function {
                        self.diags.error(0, "not a function", func);
                    }
                    
                    for arg in args {
                        self.check_expr(arg);
                    }
                    
                    sym.ty
                } else {
                    self.diags.error(0, "undeclared function", func);
                    ClType::VOID
                }
            }
            _ => ClType::VOID,
        }
    }
}