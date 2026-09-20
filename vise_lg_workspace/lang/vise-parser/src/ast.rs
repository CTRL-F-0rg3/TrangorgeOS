#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Span {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Bool,
    Int,
    Uint,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Sampler2D,
    Image2D,
    Custom(String),
    Array(Box<Type>, usize),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub qualifier: ParamQualifier,
}

#[derive(Debug, Clone, Copy)]
pub enum ParamQualifier {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String),
    Ident(String),
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
    Member { object: Box<Expr>, field: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    Cast { ty: Type, expr: Box<Expr> },
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg, Not, BitNot,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Type,
        init: Expr,
        neg_range: Option<Expr>,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    When {
        target: String,
        cases: Vec<WhenCase>,
    },
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub struct WhenCase {
    pub pattern: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StaticData {
    pub name: String,
    pub ty: Type,
    pub count: usize,
    pub init: i64,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(Function),
    Struct(StructDef),
    Enum(EnumDef),
    Static(StaticData),
}

#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<TopLevel>,
}