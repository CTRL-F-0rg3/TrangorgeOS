use crate::ir::{BinOp, CmpOp, Ir, Val};
use std::collections::HashMap;

struct FnChunk {
    name: String,
    args: usize,
    is_main: bool,
    ops: Vec<Ir>,
}

fn val(v: &Val) -> String {
    match v {
        Val::Reg(n) => n.clone(),
        Val::Imm(v) => format!("{}ull", v),
    }
}

fn binop(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::And => "&",
        BinOp::Or => "|",
        BinOp::Xor => "^",
    }
}

fn cmpop(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::NotEq => "!=",
    }
}

fn is_param(name: &str, args: usize) -> bool {
    if !name.starts_with('r') {
        return false;
    }
    match name[1..].parse::<usize>() {
        Ok(i) => i < args,
        Err(_) => false,
    }
}

fn split(ir: &[Ir]) -> Vec<FnChunk> {
    let mut fns: Vec<FnChunk> = Vec::new();
    for op in ir {
        match op {
            Ir::FnStart { name, args, is_main } => fns.push(FnChunk {
                name: name.clone(),
                args: *args,
                is_main: *is_main,
                ops: Vec::new(),
            }),
            other => {
                if let Some(f) = fns.last_mut() {
                    f.ops.push(other.clone());
                }
            }
        }
    }
    fns
}

pub fn emit(ir: &[Ir]) -> String {
    let mut out = String::new();
    out.push_str("#include <stdint.h>\n");
    out.push_str("#include <string.h>\n\n");

    let mut mem_len: HashMap<String, u64> = HashMap::new();
    for op in ir {
        if let Ir::MemDecl { name, len, .. } = op {
            mem_len.insert(name.clone(), *len);
            out.push_str(&format!("static uint8_t {}[{}];\n", name, len));
        }
    }
    out.push('\n');

    let fns = split(ir);

    for f in &fns {
        if f.is_main {
            continue;
        }
        let mut params = String::new();
        for i in 0..f.args {
            if i > 0 {
                params.push_str(", ");
            }
            params.push_str(&format!("uint64_t p{}", i));
        }
        out.push_str(&format!("static uint64_t {}({});\n", f.name, params));
    }
    out.push('\n');

    for f in &fns {
        let mut params = String::new();
        for i in 0..f.args {
            if i > 0 {
                params.push_str(", ");
            }
            params.push_str(&format!("uint64_t p{}", i));
        }

        if f.is_main {
            out.push_str("int main(void)\n{\n");
        } else {
            out.push_str(&format!("static uint64_t {}({})\n{{\n", f.name, params));
        }

        if !f.is_main {
            for i in 0..f.args {
                out.push_str(&format!("    uint64_t r{} = p{};\n", i, i));
            }
        }

        for op in &f.ops {
            if let Ir::RegDecl { name, .. } = op {
                if is_param(name, f.args) {
                    continue;
                }
                out.push_str(&format!("    uint64_t {} = 0;\n", name));
            }
        }

        for op in &f.ops {
            match op {
                Ir::Label(l) => out.push_str(&format!("{}:\n", l)),
                Ir::RegDecl { .. } | Ir::MemDecl { .. } | Ir::FnStart { .. } => {}
                Ir::SetImm { dst, imm } => {
                    out.push_str(&format!("    {} = {}ull;\n", dst, imm));
                }
                Ir::Move { dst, src } => {
                    out.push_str(&format!("    {} = {};\n", dst, val(src)));
                }
                Ir::Bin { op, dst, a, b } => {
                    out.push_str(&format!("    {} = {} {} {};\n", dst, val(a), binop(*op), val(b)));
                }
                Ir::MemFill { name, src } => {
                    let len = mem_len.get(name).copied().unwrap_or(0);
                    out.push_str(&format!("    memset({}, (int){}, {});\n", name, val(src), len));
                }
                Ir::StoreMem { name, idx, src } => {
                    out.push_str(&format!("    {}[{}] = (uint8_t){};\n", name, idx, val(src)));
                }
                Ir::LoadMem { dst, name, idx } => {
                    out.push_str(&format!("    {} = {}[{}];\n", dst, name, idx));
                }
                Ir::Branch { lhs, op, rhs, target } => {
                    out.push_str(&format!("    if ({} {} {}) goto {};\n", val(lhs), cmpop(*op), val(rhs), target));
                }
                Ir::Jump(t) => out.push_str(&format!("    goto {};\n", t)),
                Ir::Call { dst, func, args } => {
                    let mut s = String::new();
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        s.push_str(&val(a));
                    }
                    out.push_str(&format!("    {} = {}({});\n", dst, func, s));
                }
                Ir::Ret(v) => {
                    if f.is_main {
                        out.push_str(&format!("    return (int){};\n", val(v)));
                    } else {
                        out.push_str(&format!("    return {};\n", val(v)));
                    }
                }
            }
        }

        out.push_str("}\n\n");
    }

    out
}