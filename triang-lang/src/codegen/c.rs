use crate::ir::{BinOp, CmpOp, Ir, Val};
use std::collections::HashMap;

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

    out.push_str("\nint main(void)\n{\n");

    for op in ir {
        if let Ir::RegDecl { name, .. } = op {
            out.push_str(&format!("    uint64_t {} = 0;\n", name));
        }
    }

    for op in ir {
        match op {
            Ir::Label(l) => out.push_str(&format!("{}:\n", l)),
            Ir::RegDecl { .. } | Ir::MemDecl { .. } => {}
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
            Ir::Ret(v) => out.push_str(&format!("    return (int){};\n", val(v))),
        }
    }

    out.push_str("}\n");
    out
}