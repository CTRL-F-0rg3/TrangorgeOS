mod ast;
mod codegen;
mod ir;
mod iso;
mod lexer;
mod parser;
mod project;
mod sema;
mod token;

use ir::Lower;
use lexer::Lexer;
use parser::Parser;
use project::Project;
use sema::Sema;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.get(0).map(|s| s.as_str()) {
        Some("new") => cmd_new(&args[1..]),
        Some("build") => cmd_build(&args[1..]),
        Some("iso") => cmd_iso(&args[1..]),
        _ => cmd_file(&args),
    }
}

fn read_libs(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let mut paths: Vec<String> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "xl").unwrap_or(false))
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    paths.sort();
    for p in paths {
        if let Ok(s) = std::fs::read_to_string(&p) {
            out.push(s);
        }
    }
    out
}

fn pipeline(srcs: &[String]) -> Vec<ir::Ir> {
    let mut functions = Vec::new();

    for s in srcs {
        let tokens = match Lexer::new(s).tokenize() {
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

        functions.extend(program.functions);
    }

    let program = ast::Program { functions };

    if let Err(e) = Sema::new().check(&program) {
        eprintln!("SEMA ERROR: {}", e.msg);
        std::process::exit(1);
    }

    Lower::new().lower_program(&program)
}

fn cmd_new(args: &[String]) {
    let name = match args.get(0) {
        Some(n) => n.clone(),
        None => {
            eprintln!("uzycie: triang new <nazwa>");
            std::process::exit(1);
        }
    };

    let root = std::path::Path::new(&name);
    if root.exists() {
        eprintln!("katalog '{}' juz istnieje", name);
        std::process::exit(1);
    }

    std::fs::create_dir_all(root.join("src")).ok();

    let proj = format!(
        "name = {}\ntarget = x86_64\nentry = src/main.xl\nroute = asm\n",
        name
    );
    std::fs::write(root.join("triang.project"), proj).ok();

    let main_xl = "Pub fn main(u32; self) {\n    Reg u64 r0;\n    r0::set(42);\n    return r0;\n}\n";
    std::fs::write(root.join("src/main.xl"), main_xl).ok();

    println!("utworzono projekt '{}'", name);
    println!("  {}/triang.project", name);
    println!("  {}/src/main.xl", name);
}

fn cmd_build(args: &[String]) {
    let mut want_bin = false;
    let mut want_elf = false;

    for a in args {
        match a.as_str() {
            "--bin" => want_bin = true,
            "--elf" => want_elf = true,
            _ => {}
        }
    }

    if !want_bin && !want_elf {
        want_bin = true;
        want_elf = true;
    }

    let proj = match Project::load("triang.project") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("brak triang.project - uruchom 'triang new <nazwa>'");
            std::process::exit(1);
        }
    };

    let src = match std::fs::read_to_string(&proj.entry) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nie moge czytac {}: {}", proj.entry, e);
            std::process::exit(1);
        }
    };

    let mut srcs = read_libs("lib");
    srcs.push(src);
    let ir = pipeline(&srcs);

    let dir = format!("out/{}", proj.target);
    std::fs::create_dir_all(&dir).ok();

    println!("budowanie '{}' [target={} route={}]", proj.name, proj.target, proj.route);

    let elf = format!("{}/{}.elf", dir, proj.name);
    let bin = format!("{}/{}.bin", dir, proj.name);

    match proj.route.as_str() {
        "c" => {
            route_c(&ir, &format!("{}/{}.c", dir, proj.name), &elf, &bin, want_elf, want_bin);
        }
        _ => {
            route_asm(
                &ir,
                &format!("{}/{}.asm", dir, proj.name),
                &format!("{}/{}.o", dir, proj.name),
                &elf,
                &bin,
                want_elf,
                want_bin,
            );
        }
    }
}

fn cmd_file(args: &[String]) {
    let mut input = String::from("examples/sample.xl");
    let mut emit = String::from("all");
    let mut target = String::from("x86_64");
    let mut out_dir = String::from("out");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--emit" => {
                emit = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--target" => {
                target = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--out" => {
                out_dir = args.get(i + 1).cloned().unwrap_or_default();
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

    let mut srcs = read_libs("lib");
    srcs.push(src);
    let ir = pipeline(&srcs);

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
        route_c(
            &ir,
            &format!("{}/{}.c", dir, stem),
            &format!("{}/{}.c.elf", dir, stem),
            &format!("{}/{}.c.bin", dir, stem),
            true,
            true,
        );
    }

    if do_asm {
        route_asm(
            &ir,
            &format!("{}/{}.asm", dir, stem),
            &format!("{}/{}.o", dir, stem),
            &format!("{}/{}.asm.elf", dir, stem),
            &format!("{}/{}.asm.bin", dir, stem),
            true,
            true,
        );
    }
}

fn cmd_iso(_args: &[String]) {
    let proj = match Project::load("triang.project") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let bin = format!("out/{}/{}.bin", proj.target, proj.name);
    let data = match std::fs::read(&bin) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("nie moge czytac {} (najpierw 'triang build --bin'): {}", bin, e);
            std::process::exit(1);
        }
    };

    let iso = iso::build(
        "TRANGORGE",
        &[iso::IsoFile {
            name: "KERNEL.BIN;1".to_string(),
            data,
        }],
    );

    let out = format!("out/{}.iso", proj.name);
    std::fs::create_dir_all("out").ok();
    std::fs::write(&out, iso).ok();
    println!("-> {}", out);
}

fn route_c(ir: &[ir::Ir], text_path: &str, elf_path: &str, bin_path: &str, want_elf: bool, want_bin: bool) {
    std::fs::write(text_path, codegen::c::emit(ir)).ok();
    println!("-> {}", text_path);

    if !want_elf && !want_bin {
        return;
    }

    match Command::new("cc").arg(text_path).arg("-o").arg(elf_path).status() {
        Ok(s) if s.success() => {
            if want_elf {
                println!("-> {}", elf_path);
            }
            if want_bin && run_objcopy(elf_path, bin_path) {
                println!("-> {}", bin_path);
            }
        }
        _ => eprintln!("cc: niedostepny lub blad"),
    }
}

fn route_asm(
    ir: &[ir::Ir],
    text_path: &str,
    obj_path: &str,
    elf_path: &str,
    bin_path: &str,
    want_elf: bool,
    want_bin: bool,
) {
    std::fs::write(text_path, codegen::asm::emit(ir)).ok();
    println!("-> {}", text_path);

    if !want_elf && !want_bin {
        return;
    }

    let ok_nasm = Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg(text_path)
        .arg("-o")
        .arg(obj_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok_nasm {
        eprintln!("nasm: niedostepny lub blad");
        return;
    }

    if Command::new("gcc")
        .arg(obj_path)
        .arg("-o")
        .arg(elf_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        if want_elf {
            println!("-> {}", elf_path);
        }
        if want_bin && run_objcopy(elf_path, bin_path) {
            println!("-> {}", bin_path);
        }
    }
}

fn run_objcopy(elf: &str, bin: &str) -> bool {
    Command::new("objcopy")
        .arg("-O")
        .arg("binary")
        .arg(elf)
        .arg(bin)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}