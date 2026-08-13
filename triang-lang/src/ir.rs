pub use crate::ast::*;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Reg(String),
    Imm(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ir {
    RegDecl { name: String, ty: Type },
    MemDecl { name: String, ty: Type, len: u64 },
    Label(String),
    SetImm { dst: String, imm: u64 },
    Move { dst: String, src: Val },
    Bin { op: BinOp, dst: String, a: Val, b: Val },
    MemFill { name: String, src: Val },
    StoreMem { name: String, idx: u64, src: Val },
    LoadMem { dst: String, name: String, idx: u64 },
    Branch { lhs: Val, op: CmpOp, rhs: Val, target: String },
    Jump(String),
    Ret(Val),
    FnStart { name: String, args: usize, is_main: bool },
    Call { dst: String, func: String, args: Vec<Val> },
}

pub struct Lower {
    out: Vec<Ir>,
    counter: usize,
    mems: HashSet<String>,
}

impl Lower {
    pub fn new() -> Self {
        Self {
            out: Vec::new(),
            counter: 0,
            mems: HashSet::new(),
        }
    }

    fn label(&mut self, tag: &str) -> String {
        self.counter += 1;
        format!("{}_{}", tag, self.counter)
    }

    pub fn lower_program(mut self, program: &Program) -> Vec<Ir> {
        for f in &program.functions {
            self.lower_function(f);
        }
        self.out
    }

    fn lower_function(&mut self, f: &Function) {
        let args = f.params.iter().filter(|p| matches!(p, Param::Typed(_))).count();
        let is_main = f.name == "main";
        self.out.push(Ir::FnStart {
            name: f.name.clone(),
            args,
            is_main,
        });
        for s in &f.body {
            self.lower_stmt(s);
        }
    }

    fn lower_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::ConstDecl { .. } | Stmt::StaticDecl { .. } => {}
            Stmt::RegDecl { ty, name } => {
                self.out.push(Ir::RegDecl { name: name.clone(), ty: *ty });
            }
            Stmt::MemDecl { ty, len, name } => {
                self.mems.insert(name.clone());
                self.out.push(Ir::MemDecl { name: name.clone(), ty: *ty, len: *len });
            }
            Stmt::Op(call) => self.lower_op(call),
            Stmt::If { cond, then_body, else_body } => {
                let lelse = self.label("else");
                let lend = self.label("end");
                self.out.push(Ir::Branch {
                    lhs: val(&cond.lhs),
                    op: invert(cond.op),
                    rhs: val(&cond.rhs),
                    target: lelse.clone(),
                });
                for s in then_body {
                    self.lower_stmt(s);
                }
                self.out.push(Ir::Jump(lend.clone()));
                self.out.push(Ir::Label(lelse));
                for s in else_body {
                    self.lower_stmt(s);
                }
                self.out.push(Ir::Label(lend));
            }
            Stmt::While { cond, body } => {
                let lstart = self.label("loop");
                let lend = self.label("loop_end");
                self.out.push(Ir::Label(lstart.clone()));
                self.out.push(Ir::Branch {
                    lhs: val(&cond.lhs),
                    op: invert(cond.op),
                    rhs: val(&cond.rhs),
                    target: lend.clone(),
                });
                for s in body {
                    self.lower_stmt(s);
                }
                self.out.push(Ir::Jump(lstart));
                self.out.push(Ir::Label(lend));
            }
            Stmt::Return(e) => {
                let v = match e {
                    Some(e) => val(e),
                    None => Val::Imm(0),
                };
                self.out.push(Ir::Ret(v));
            }
        }
    }

    fn lower_op(&mut self, call: &OpCall) {
        
        match &call.target {
            Target::Named(dst) => {
                if self.mems.contains(dst) {
                    match call.op.as_str() {
                        "call" => {
                            let func = match &call.args[0] {
                                Expr::Ident(n) => n.clone(),
                                _ => String::new(),
                            };
                            self.out.push(Ir::Call {
                                dst: dst.clone(),
                                func,
                                args: call.args[1..].iter().map(val).collect(),
                            });
                        }
                        "set" => {
                            self.out.push(Ir::MemFill {
                                name: dst.clone(),
                                src: val(&call.args[0]),
                            });
                        }
                        _ => {}
                    }
                } else {
                    match call.op.as_str() {
                        "set" => match &call.args[0] {
                            Expr::Int(v) => {
                                self.out.push(Ir::SetImm { dst: dst.clone(), imm: *v });
                            }
                            other => {
                                self.out.push(Ir::Move { dst: dst.clone(), src: val(other) });
                            }
                        },
                        "move" => {
                            self.out.push(Ir::Move { dst: dst.clone(), src: val(&call.args[0]) });
                        }
                        "load" => match &call.args[0] {
                            Expr::Indexed(name, idx) => {
                                self.out.push(Ir::LoadMem {
                                    dst: dst.clone(),
                                    name: name.clone(),
                                    idx: *idx,
                                });
                            }
                            _ => {}
                        },
                        op => {
                            self.out.push(Ir::Bin {
                                op: binop(op),
                                dst: dst.clone(),
                                a: val(&call.args[0]),
                                b: val(&call.args[1]),
                            });
                        }
                    }
                }
            }
            Target::Indexed(name, idx) => {
                self.out.push(Ir::StoreMem {
                    name: name.clone(),
                    idx: *idx,
                    src: val(&call.args[0]),
                });
            }
            Target::Region => {}
        }
    }
}

fn val(e: &Expr) -> Val {
    match e {
        Expr::Int(v) => Val::Imm(*v),
        Expr::Ident(n) => Val::Reg(n.clone()),
        Expr::Indexed(_, _) => Val::Imm(0),
    }
}

fn invert(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Eq => CmpOp::NotEq,
        CmpOp::NotEq => CmpOp::Eq,
    }
}

fn binop(s: &str) -> BinOp {
    match s {
        "add" => BinOp::Add,
        "sub" => BinOp::Sub,
        "mul" => BinOp::Mul,
        "div" => BinOp::Div,
        "and" => BinOp::And,
        "or" => BinOp::Or,
        "xor" => BinOp::Xor,
        _ => BinOp::Add,
    }
}