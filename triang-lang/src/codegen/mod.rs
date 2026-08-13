pub mod asm;
pub mod c;
pub mod target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emit {
    C,
    Asm,
    Bin,
    Elf,
}