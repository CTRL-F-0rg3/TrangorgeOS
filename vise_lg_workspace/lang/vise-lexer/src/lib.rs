// vise_lg_workspace/lang/vise-lexer/src/lib.rs
#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Token kinds of the Vise LG (graphics language) surface syntax.
///
/// NOTE: `Eq` cannot be derived — `FloatLiteral` carries an `f64`.
/// `TokenKind::Ident` carries the identifier text so the parser and the
/// code generator can use real names.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn, Let, Mut, Return, If, Else, For, While,
    Struct, Enum, Match, Static, When,
    In, Out, InOut,
    Vec2, Vec3, Vec4, Mat3, Mat4,
    Float, Int, Uint, Bool,
    Sampler2D, Image2D,

    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),

    // Identifiers
    Ident(String),
    Underscore,

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    Assign, Arrow, EqColon, Tilde,

    // Delimiters
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Semicolon, Colon, Dot,

    // Special
    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
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
            b'-' => {
                self.advance();
                if self.eat(b'>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            b'*' => { self.advance(); TokenKind::Star }
            b'/' => { self.advance(); TokenKind::Slash }
            b'%' => { self.advance(); TokenKind::Percent }
            b'~' => { self.advance(); TokenKind::Tilde }

            b'=' => {
                self.advance();
                if self.eat(b'=') { TokenKind::Eq } else { TokenKind::Assign }
            }
            b'!' => {
                self.advance();
                if self.eat(b'=') { TokenKind::Ne } else { TokenKind::Not }
            }
            b'<' => {
                self.advance();
                if self.eat(b'=') { TokenKind::Le } else { TokenKind::Lt }
            }
            b'>' => {
                self.advance();
                if self.eat(b'=') { TokenKind::Ge } else { TokenKind::Gt }
            }
            b'&' => {
                self.advance();
                if self.eat(b'&') { TokenKind::And } else { TokenKind::Error }
            }
            b'|' => {
                self.advance();
                if self.eat(b'|') { TokenKind::Or } else { TokenKind::Error }
            }

            b'(' => { self.advance(); TokenKind::LParen }
            b')' => { self.advance(); TokenKind::RParen }
            b'{' => { self.advance(); TokenKind::LBrace }
            b'}' => { self.advance(); TokenKind::RBrace }
            b'[' => { self.advance(); TokenKind::LBracket }
            b']' => { self.advance(); TokenKind::RBracket }
            b',' => { self.advance(); TokenKind::Comma }
            b';' => { self.advance(); TokenKind::Semicolon }
            b':' => {
                self.advance();
                if self.eat(b'=') { TokenKind::EqColon } else { TokenKind::Colon }
            }
            b'.' => { self.advance(); TokenKind::Dot }

            b'"' => self.lex_string(),

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
    
    /// Consume the byte at the cursor if it equals `expected`.
    fn eat(&mut self, expected: u8) -> bool {
        if self.pos < self.source.len() && self.source[self.pos] == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skip whitespace, `// line comments` and `#directive` lines.
    fn skip_whitespace(&mut self) {
        loop {
            while self.pos < self.source.len() {
                let c = self.source[self.pos];
                if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                    self.advance();
                } else {
                    break;
                }
            }

            // Line comment / preprocessor-style directive.
            let is_comment = self.pos + 1 < self.source.len()
                && self.source[self.pos] == b'/'
                && self.source[self.pos + 1] == b'/';
            let is_directive = self.pos < self.source.len() && self.source[self.pos] == b'#';

            if is_comment || is_directive {
                while self.pos < self.source.len() && self.source[self.pos] != b'\n' {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn lex_string(&mut self) -> TokenKind {
        self.advance(); // opening quote
        let start = self.pos;

        while self.pos < self.source.len() && self.source[self.pos] != b'"' {
            self.advance();
        }

        let text = core::str::from_utf8(&self.source[start..self.pos]).unwrap_or("");
        let kind = TokenKind::StringLiteral(String::from(text));
        self.eat(b'"'); // closing quote
        kind
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
        
        let text = core::str::from_utf8(&self.source[start..self.pos]).unwrap_or("");

        match text {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "match" => TokenKind::Match,
            "static" => TokenKind::Static,
            "when" => TokenKind::When,
            "vec2" => TokenKind::Vec2,
            "vec3" => TokenKind::Vec3,
            "vec4" => TokenKind::Vec4,
            "mat3" => TokenKind::Mat3,
            "mat4" => TokenKind::Mat4,
            "float" => TokenKind::Float,
            "int" => TokenKind::Int,
            "uint" => TokenKind::Uint,
            "bool" => TokenKind::Bool,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            "sampler2D" => TokenKind::Sampler2D,
            "image2D" => TokenKind::Image2D,
            "in" => TokenKind::In,
            "out" => TokenKind::Out,
            "inout" => TokenKind::InOut,
            "_" => TokenKind::Underscore,
            _ => TokenKind::Ident(String::from(text)),
        }
    }

    /// Lex the whole source into a token vector (no trailing `Eof` token).
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
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

/// Convenience wrapper: lex `source` into a token vector (no trailing `Eof`).
pub fn tokenize(source: &[u8]) -> Vec<Token> {
    Lexer::new(source).tokenize()
}

/// Self-test used by the kernel and by the host-side verifier.
///
/// Returns a human readable description on success, an error message otherwise.
/// The signature (`Result<&'static str, &'static str>`) matches the kernel test
/// harness (`kernel::testing::TestResult`) so the function can be registered
/// directly in `kernel_Workspace/kernel/src/main.rs`.
pub fn self_test() -> Result<&'static str, &'static str> {
    let tokens = tokenize(b"fn main(vec3 p) -> vec4 { let vec4 c = vec4(p, 1.0); return c; }");

    if tokens.len() < 10 {
        return Err("vise-lexer: token stream too short");
    }

    if tokens[0].kind != TokenKind::Fn {
        return Err("vise-lexer: expected `fn` keyword");
    }

    match &tokens[1].kind {
        TokenKind::Ident(name) if name == "main" => {}
        _ => return Err("vise-lexer: identifier text lost"),
    }

    if tokens[2].kind != TokenKind::LParen || tokens[3].kind != TokenKind::Vec3 {
        return Err("vise-lexer: expected `(vec3`");
    }

    // `->` must be a single Arrow token, not Minus + Gt.
    if !tokens.iter().any(|t| t.kind == TokenKind::Arrow) {
        return Err("vise-lexer: `->` not lexed as Arrow");
    }

    if !tokens.iter().any(|t| t.kind == TokenKind::FloatLiteral(1.0)) {
        return Err("vise-lexer: float literal not lexed");
    }

    // Comments and directives are skipped, not tokenised.
    let commented = tokenize(b"#version 1\n// shader\nfn f() -> float { return 1.0; }");
    if commented[0].kind != TokenKind::Fn {
        return Err("vise-lexer: comment/directive skipping broken");
    }

    Ok("vise lexer: keywords/idents/literals/ops (incl. -> and comments)")
}
