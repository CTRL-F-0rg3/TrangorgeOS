mod ast;
mod codegen;
mod ir;
mod lexer;
mod parser;
mod sema;
mod token;

use ir::Lower;
use lexer::Lexer;
use parser::Parser;
use sema::Sema;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut input = String::from("examples/sample.tlx");
    let mut emit = String::from("all");
    let mut target = String::from("x86_64");
    let mut out_dir = String::from("out");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--emit" => {
                if let Some(v) = args.get(i + 1) {
                    emit = v.clone();
                }
                i += 2;
            }
            "--target" => {
                if let Some(v) = args.get(i + 1) {
                    target = v.clone();
                }
                i += 2;
            }
            "--out" => {
                if let Some(v) = args.get(i + 1) {
                    out_dir = v.clone();
                }
                i += 2;
            }
            other => {
                input = other.to_string();
                i += 1;
            }
        }
    }

    let src = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nie moge czytac {}: {}", input, e);
            std::process::exit(1);
        }
    };

    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("LEX ERROR @ {}:{}: {}", e.line, e.col, e.msg);
            std::process::exit(1);
        }
    };

    let program = match Parser::new(tokens).parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("PARSE ERROR @ {}:{}: {}", e.line, e.col, e.msg);
            std::process::exit(1);
        }
    };

    if let Err(e) = Sema::new().check(&program) {
        eprintln!("SEMA ERROR: {}", e.msg);
        std::process::exit(1);
    }

    let ir = Lower::new().lower_program(&program);

    let stem = input
        .rsplit('/')
        .next()
        .unwrap_or("prog")
        .strip_suffix(".xl")
        .unwrap_or("prog")
        .to_string();

    let dir = format!("{}/{}", out_dir, target);
    std::fs::create_dir_all(&dir).ok();

    let do_c = emit == "all" || emit == "c";
    let do_asm = emit == "all" || emit == "asm";

    if do_c {
        let c_path = format!("{}/{}.c", dir, stem);
        std::fs::write(&c_path, codegen::c::emit(&ir)).ok();
        println!("-> {}", c_path);

        let elf = format!("{}/{}.c.elf", dir, stem);
        let bin = format!("{}/{}.c.bin", dir, stem);

        match Command::new("cc").arg(&c_path).arg("-o").arg(&elf).status() {
            Ok(s) if s.success() => {
                println!("-> {}", elf);
                if Command::new("objcopy")
                    .arg("-O")
                    .arg("binary")
                    .arg(&elf)
                    .arg(&bin)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    println!("-> {}", bin);
                }
            }
            _ => eprintln!("cc: niedostepny lub blad"),
        }
    }

    if do_asm {
        let asm_path = format!("{}/{}.asm", dir, stem);
        std::fs::write(&asm_path, codegen::asm::emit(&ir)).ok();
        println!("-> {}", asm_path);

        let obj = format!("{}/{}.o", dir, stem);
        let elf = format!("{}/{}.asm.elf", dir, stem);
        let bin = format!("{}/{}.asm.bin", dir, stem);

        let ok_nasm = Command::new("nasm")
            .arg("-f")
            .arg("elf64")
            .arg(&asm_path)
            .arg("-o")
            .arg(&obj)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if ok_nasm {
            if Command::new("gcc")
                .arg(&obj)
                .arg("-o")
                .arg(&elf)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                println!("-> {}", elf);
                if Command::new("objcopy")
                    .arg("-O")
                    .arg("binary")
                    .arg(&elf)
                    .arg(&bin)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    println!("-> {}", bin);
                }
            }
        } else {
            eprintln!("nasm: niedostepny lub blad");
        }
    }
}