// vise_lg_workspace/tools/vise-compiler/src/main.rs
use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: vise-compiler <input.vise> <output.vb>");
        return;
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };
    
    let bytecode = match compile(&source) {
        Some(bc) => bc,
        None => {
            eprintln!("Compilation failed");
            return;
        }
    };
    
    if let Err(e) = fs::write(output_path, bytecode) {
        eprintln!("Error writing output: {}", e);
    }
}

fn compile(source: &str) -> Option<Vec<u8>> {
    let mut lexer = vise_lexer::Lexer::new(source.as_bytes());
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