#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub is_pub: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Typed(Type),
    SelfParam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub ty: Type,
    pub count: u64,
    pub shift: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ConstDecl { from: Layout, to: Layout },
    StaticDecl {
        dims: (u64, u64),
        format: String,
        map_to: u64,
        op: OpCall,
    },
    RegDecl { ty: Type, name: String },
    MemDecl { ty: Type, len: u64, name: String },
    Op(OpCall),
    If {
        cond: Cond,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    While { cond: Cond, body: Vec<Stmt> },
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpCall {
    pub target: Target,
    pub op: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Named(String),
    Indexed(String, u64),
    Region,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(u64),
    Ident(String),
    Indexed(String, u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cond {
    pub lhs: Expr,
    pub op: CmpOp,
    pub rhs: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    NotEq,
}