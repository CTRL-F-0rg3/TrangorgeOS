use crate::codegen::target::Target;
use crate::ir::{BinOp, CmpOp, Ir, Val};

const SCRATCH_A: &str = "r14";
const SCRATCH_B: &str = "r15";
const SCRATCH_C: &str = "r12";

struct Emitter {
    out: String,
    counter: usize,
}

impl Emitter {
    fn label(&mut self, tag: &str) -> String {
        self.counter += 1;
        format!("{}_{}", tag, self.counter)
    }

    fn line(&mut self, s: &str) {
        self.out.push_str("    ");
        self.out.push_str(s);
        self.out.push('\n');
    }
}

fn op(t: &Target, v: &Val) -> String {
    match v {
        Val::Reg(n) => t.reg(n).to_string(),
        Val::Imm(v) => v.to_string(),
    }
}

fn mem_len(ir: &[Ir], name: &str) -> u64 {
    for op_ in ir {
        if let Ir::MemDecl { name: n, len, .. } = op_ {
            if n == name {
                return *len;
            }
        }
    }
    0
}

pub fn emit(ir: &[Ir]) -> String {
    let t = Target::X86_64;
    let mut e = Emitter { out: String::new(), counter: 0 };

    e.out.push_str("bits 64\n\n");

    e.out.push_str("section .bss\n");
    for op_ in ir {
        if let Ir::MemDecl { name, len, .. } = op_ {
            e.out.push_str(&format!("{}: resb {}\n", name, len));
        }
    }
    e.out.push('\n');

    e.out.push_str("section .text\n");

    for op_ in ir {
        match op_ {
            Ir::FnStart { name, is_main, .. } => {
                if *is_main {
                    e.out.push_str("global main\n");
                    e.out.push_str("main:\n");
                }
                e.out.push_str(&format!("fn_{}:\n", name));
            }
            Ir::Label(l) => {
                e.out.push_str(&format!("{}:\n", l));
            }
            Ir::RegDecl { .. } | Ir::MemDecl { .. } => {}
            Ir::SetImm { dst, imm } => {
                e.line(&format!("mov {}, {}", t.reg(dst), imm));
            }
            Ir::Move { dst, src } => {
                e.line(&format!("mov {}, {}", t.reg(dst), op(&t, src)));
            }
            Ir::Bin { op: bop, dst, a, b } => {
                let d = t.reg(dst).to_string();
                let a_s = op(&t, a);
                let b_s = op(&t, b);
                match bop {
                    BinOp::Div => {
                        e.line(&format!("mov rax, {}", a_s));
                        e.line("xor rdx, rdx");
                        e.line(&format!("div {}", b_s));
                        e.line(&format!("mov {}, rax", d));
                    }
                    _ => {
                        if d != a_s {
                            e.line(&format!("mov {}, {}", d, a_s));
                        }
                        let ins = match bop {
                            BinOp::Add => "add",
                            BinOp::Sub => "sub",
                            BinOp::And => "and",
                            BinOp::Or => "or",
                            BinOp::Xor => "xor",
                            BinOp::Mul => "imul",
                            BinOp::Div => unreachable!(),
                        };
                        e.line(&format!("{} {}, {}", ins, d, b_s));
                    }
                }
            }
            Ir::MemFill { name, src } => {
                let lstart = e.label("fill");
                let lend = e.label("fill_end");
                e.line(&format!("lea {}, [rel {}]", SCRATCH_A, name));
                e.line(&format!("mov {}, {}", SCRATCH_B, mem_len(ir, name)));
                e.line(&format!("mov {}, {}", SCRATCH_C, op(&t, src)));
                e.out.push_str(&format!("{}:\n", lstart));
                e.line(&format!("test {}, {}", SCRATCH_B, SCRATCH_B));
                e.line(&format!("jz {}", lend));
                e.line(&format!("mov byte [{}], {}b", SCRATCH_A, SCRATCH_C));
                e.line(&format!("inc {}", SCRATCH_A));
                e.line(&format!("dec {}", SCRATCH_B));
                e.line(&format!("jmp {}", lstart));
                e.out.push_str(&format!("{}:\n", lend));
            }
            Ir::StoreMem { name, idx, src } => {
                e.line(&format!("lea {}, [rel {}]", SCRATCH_A, name));
                match src {
                    Val::Imm(v) => {
                        e.line(&format!("mov byte [{} + {}], {}", SCRATCH_A, idx, v));
                    }
                    Val::Reg(n) => {
                        e.line(&format!("mov {}, {}", SCRATCH_B, t.reg(n)));
                        e.line(&format!("mov byte [{} + {}], {}b", SCRATCH_A, idx, SCRATCH_B));
                    }
                }
            }
            Ir::LoadMem { dst, name, idx } => {
                e.line(&format!("lea {}, [rel {}]", SCRATCH_A, name));
                e.line(&format!("movzx {}, byte [{} + {}]", t.reg(dst), SCRATCH_A, idx));
            }
            Ir::Branch { lhs, op: cop, rhs, target } => {
                match lhs {
                    Val::Reg(n) => {
                        e.line(&format!("cmp {}, {}", t.reg(n), op(&t, rhs)));
                    }
                    Val::Imm(v) => {
                        e.line(&format!("mov {}, {}", SCRATCH_A, v));
                        e.line(&format!("cmp {}, {}", SCRATCH_A, op(&t, rhs)));
                    }
                }
                let ins = match cop {
                    CmpOp::Eq => "je",
                    CmpOp::NotEq => "jne",
                };
                e.line(&format!("{} {}", ins, target));
            }
            Ir::Jump(target) => {
                e.line(&format!("jmp {}", target));
            }
            Ir::Call { dst, func, args } => {
                let scr = ["r12", "r13", "r14", "r15"];
                let argr = ["rax", "rbx", "rcx", "rdx"];
                for (i, a) in args.iter().enumerate().take(4) {
                    e.line(&format!("mov {}, {}", scr[i], op(&t, a)));
                }
                for i in 0..args.len().min(4) {
                    e.line(&format!("mov {}, {}", argr[i], scr[i]));
                }
                e.line(&format!("call fn_{}", func));
                let d = t.reg(dst);
                if d != "rax" {
                    e.line(&format!("mov {}, rax", d));
                }
            }
            Ir::FOpen { dst, path, mode } => {
                e.line(&format!("lea rdi, [rel {}]", path));
                if *mode != 0 {
                    e.line("mov rsi, 0x41");
                    e.line("mov rdx, 0x1A4");
                } else {
                    e.line("xor rsi, rsi");
                    e.line("xor rdx, rdx");
                }
                e.line("mov rax, 2");
                e.line("syscall");
                e.line(&format!("mov {}, rax", t.reg(dst)));
            }
            Ir::FWrite { dst, fd, buf, len } => {
                e.line(&format!("mov rdi, {}", op(&t, fd)));
                e.line(&format!("lea rsi, [rel {}]", buf));
                e.line(&format!("mov rdx, {}", op(&t, len)));
                e.line("mov rax, 1");
                e.line("syscall");
                e.line(&format!("mov {}, rax", t.reg(dst)));
            }
            Ir::FRead { dst, fd, buf, len } => {
                e.line(&format!("mov rdi, {}", op(&t, fd)));
                e.line(&format!("lea rsi, [rel {}]", buf));
                e.line(&format!("mov rdx, {}", op(&t, len)));
                e.line("xor rax, rax");
                e.line("syscall");
                e.line(&format!("mov {}, rax", t.reg(dst)));
            }
            Ir::FClose { dst, fd } => {
                e.line(&format!("mov rdi, {}", op(&t, fd)));
                e.line("mov rax, 3");
                e.line("syscall");
                e.line(&format!("mov {}, 0", t.reg(dst)));
            }
            Ir::Ret(v) => {
                let r = op(&t, v);
                if r != t.ret_reg() {
                    e.line(&format!("mov {}, {}", t.ret_reg(), r));
                }
                e.line("ret");
            }
        }
    }

    e.out
}