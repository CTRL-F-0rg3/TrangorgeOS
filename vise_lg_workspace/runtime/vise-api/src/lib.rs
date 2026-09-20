// vise_lg_workspace/runtime/vise-api/src/lib.rs
#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone)]
pub struct ShaderModule {
    pub stage: ShaderStage,
    pub bytecode: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PipelineLayout {
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub set: u32,
    pub binding: u32,
    pub binding_type: BindingType,
}

#[derive(Debug, Clone, Copy)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Sampler,
    Image,
}

#[derive(Debug, Clone)]
pub struct GraphicsPipeline {
    pub vertex_shader: ShaderModule,
    pub fragment_shader: ShaderModule,
    pub layout: PipelineLayout,
}

pub struct Device {
    gpu_handle: u64,
}

impl Device {
    pub fn new() -> Option<Self> {
        Some(Self { gpu_handle: 0 })
    }
    
    pub fn create_shader_module(&self, stage: ShaderStage, source: &[u8]) -> Option<ShaderModule> {
        let bytecode = compile_shader(stage, source)?;
        Some(ShaderModule { stage, bytecode })
    }
    
    pub fn create_graphics_pipeline(
        &self,
        vertex_shader: ShaderModule,
        fragment_shader: ShaderModule,
        layout: PipelineLayout,
    ) -> Option<GraphicsPipeline> {
        Some(GraphicsPipeline {
            vertex_shader,
            fragment_shader,
            layout,
        })
    }
}

fn compile_shader(stage: ShaderStage, source: &[u8]) -> Option<Vec<u8>> {
    let mut lexer = vise_lexer::Lexer::new(source);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token();
        if token.kind == vise_lexer::TokenKind::Eof {
            break;
        }
        tokens.push(token);
    }
    
    let mut parser = vise_parser::Parser::new(&tokens);
    let ast = parser.parse()?;
    
    let ir = vise_codegen::lower_to_ir(&ast)?;
    let optimized = vise_optimizer::optimize(ir)?;
    let bytecode = vise_codegen::generate_bytecode(&optimized)?;
    
    Some(bytecode)
}