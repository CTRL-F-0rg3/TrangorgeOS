// vise_lg_workspace/lang/vise-ir/src/lib.rs
#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Clone)]
pub enum Instruction {
    ConstInt(i64),
    ConstFloat(f64),
    ConstBool(bool),
    
    Add { dest: u32, a: u32, b: u32 },
    Sub { dest: u32, a: u32, b: u32 },
    Mul { dest: u32, a: u32, b: u32 },
    Div { dest: u32, a: u32, b: u32 },
    
    Load { dest: u32, ptr: u32 },
    Store { ptr: u32, value: u32 },
    
    Call { dest: u32, func: String, args: Vec<u32> },
    
    Return { value: Option<u32> },
    
    Branch { target: u32 },
    ConditionalBranch { cond: u32, true_target: u32, false_target: u32 },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: u32,
    pub instructions: Vec<Instruction>,
    pub terminator: Option<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(String, Type)>,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: Vec<Function>,
    pub globals: Vec<(String, Type)>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
        }
    }
    
    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }
}

impl BasicBlock {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: None,
        }
    }
    
    pub fn push(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }
}