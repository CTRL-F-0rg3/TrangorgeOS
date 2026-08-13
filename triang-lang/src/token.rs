// xlang/src/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Pub,
    Fn,
    Const,
    Static,
    Reg,
    Mem,
    If,
    Else,
    While,
    Return,
    SelfKw,

    TypeU8, TypeU16, TypeU32, TypeU64,
    TypeI8, TypeI16, TypeI32, TypeI64,

    Ident(String),
    Int(u64),
    Dims(u64, u64),

    ColonColon, // ::
    FatArrow,   // =>
    Arrow,      // ->
    ShiftLeft,  // <<
    EqEq,       // ==
    NotEq,      // !=

    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Semicolon,
    Comma,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}