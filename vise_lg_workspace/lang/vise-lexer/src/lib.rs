// vise_lg_workspace/lang/vise-lexer/src/lib.rs
#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fn, Let, Mut, Return, If, Else, For, While,
    Struct, Enum, Match,
    In, Out, InOut,
    Vec2, Vec3, Vec4, Mat3, Mat4,
    Float, Int, Uint, Bool,
    Sampler2D, Image2D,
    
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral,
    
    // Identifiers
    Ident,
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    Assign,
    
    // Delimiters
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Semicolon, Colon, Dot,
    
    // Special
    Eof,
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { source, pos: 0, line: 1, column: 1 }
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if self.pos >= self.source.len() {
            return Token { kind: TokenKind::Eof, span: self.current_span() };
        }
        
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;
        
        let c = self.source[self.pos];
        
        let kind = match c {
            b'+' => { self.advance(); TokenKind::Plus }
            b'-' => { self.advance(); TokenKind::Minus }
            b'*' => { self.advance(); TokenKind::Star }
            b'/' => { self.advance(); TokenKind::Slash }
            b'(' => { self.advance(); TokenKind::LParen }
            b')' => { self.advance(); TokenKind::RParen }
            b'{' => { self.advance(); TokenKind::LBrace }
            b'}' => { self.advance(); TokenKind::RBrace }
            b'[' => { self.advance(); TokenKind::LBracket }
            b']' => { self.advance(); TokenKind::RBracket }
            b',' => { self.advance(); TokenKind::Comma }
            b';' => { self.advance(); TokenKind::Semicolon }
            b':' => { self.advance(); TokenKind::Colon }
            b'.' => { self.advance(); TokenKind::Dot }
            
            b'0'..=b'9' => self.lex_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_identifier(),
            
            _ => {
                self.advance();
                TokenKind::Error
            }
        };
        
        Token {
            kind,
            span: Span {
                start,
                end: self.pos,
                line: start_line,
                column: start_col,
            },
        }
    }
    
    fn advance(&mut self) {
        if self.pos < self.source.len() {
            if self.source[self.pos] == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let c = self.source[self.pos];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn lex_number(&mut self) -> TokenKind {
        let mut is_float = false;
        let start = self.pos;
        
        while self.pos < self.source.len() {
            let c = self.source[self.pos];
            if c.is_ascii_digit() {
                self.advance();
            } else if c == b'.' && !is_float {
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start..self.pos];
        if is_float {
            let val = core::str::from_utf8(text).unwrap_or("0.0");
            TokenKind::FloatLiteral(val.parse().unwrap_or(0.0))
        } else {
            let val = core::str::from_utf8(text).unwrap_or("0");
            TokenKind::IntLiteral(val.parse().unwrap_or(0))
        }
    }
    
    fn lex_identifier(&mut self) -> TokenKind {
        let start = self.pos;
        
        while self.pos < self.source.len() {
            let c = self.source[self.pos];
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start..self.pos];
        match text {
            b"fn" => TokenKind::Fn,
            b"let" => TokenKind::Let,
            b"mut" => TokenKind::Mut,
            b"return" => TokenKind::Return,
            b"if" => TokenKind::If,
            b"else" => TokenKind::Else,
            b"for" => TokenKind::For,
            b"while" => TokenKind::While,
            b"struct" => TokenKind::Struct,
            b"vec2" => TokenKind::Vec2,
            b"vec3" => TokenKind::Vec3,
            b"vec4" => TokenKind::Vec4,
            b"mat3" => TokenKind::Mat3,
            b"mat4" => TokenKind::Mat4,
            b"float" => TokenKind::Float,
            b"int" => TokenKind::Int,
            b"uint" => TokenKind::Uint,
            b"bool" => TokenKind::Bool,
            b"sampler2D" => TokenKind::Sampler2D,
            b"image2D" => TokenKind::Image2D,
            b"in" => TokenKind::In,
            b"out" => TokenKind::Out,
            b"inout" => TokenKind::InOut,
            _ => TokenKind::Ident,
        }
    }
    
    fn current_span(&self) -> Span {
        Span {
            start: self.pos,
            end: self.pos,
            line: self.line,
            column: self.column,
        }
    }
}