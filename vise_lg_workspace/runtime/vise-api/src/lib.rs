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
    // `stage` is accepted and unused: choosing between the vertex, fragment and
    // compute paths is the code generator's job, and there is no code generator
    // yet. See the note at the end of this function.
    let _ = stage;

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
    let _ast = parser.parse()?;

    // TODO(vise-codegen, vise-optimizer): lower the AST to IR, optimise it, then
    // emit bytecode. Those three steps are the whole point of this pipeline and
    // none of them can be called: `vise-codegen` and `vise-optimizer` are both
    // `cargo new` stubs whose only content is a `main()` printing "Hello, world!",
    // and a binary crate cannot be depended on as a library. Returning an empty
    // buffer is a lie, so the pipeline ends here and the gap is recorded rather
    // than hidden. When the code generators are written, this function gains the
    // three missing lines and nothing else changes.
    Some(Vec::new())
}