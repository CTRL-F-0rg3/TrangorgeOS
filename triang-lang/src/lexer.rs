
use crate::token::{Token, TokenKind};

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if let Some(c) = c {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_ws_and_comments();

            let (line, col) = (self.line, self.col);

            let Some(c) = self.peek() else {
                tokens.push(Token { kind: TokenKind::Eof, line, col });
                break;
            };

            let kind = match c {
                '(' => { self.bump(); TokenKind::LParen }
                ')' => { self.bump(); TokenKind::RParen }
                '{' => { self.bump(); TokenKind::LBrace }
                '}' => { self.bump(); TokenKind::RBrace }
                '[' => { self.bump(); TokenKind::LBracket }
                ']' => { self.bump(); TokenKind::RBracket }
                ';' => { self.bump(); TokenKind::Semicolon }
                ',' => { self.bump(); TokenKind::Comma }

                ':' if self.peek2() == Some(':') => { self.bump(); self.bump(); TokenKind::ColonColon }
                '=' if self.peek2() == Some('>') => { self.bump(); self.bump(); TokenKind::FatArrow }
                '=' if self.peek2() == Some('=') => { self.bump(); self.bump(); TokenKind::EqEq }
                '-' if self.peek2() == Some('>') => { self.bump(); self.bump(); TokenKind::Arrow }
                '<' if self.peek2() == Some('<') => { self.bump(); self.bump(); TokenKind::ShiftLeft }
                '!' if self.peek2() == Some('=') => { self.bump(); self.bump(); TokenKind::NotEq }

                c if c.is_ascii_digit() => self.read_number()?,
                c if is_ident_start(c) => self.read_ident(),

                other => {
                    return Err(LexError {
                        msg: format!("nieznany znak '{}'", other),
                        line,
                        col,
                    })
                }
            };

            tokens.push(Token { kind, line, col });
        }

        Ok(tokens)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            match (self.peek(), self.peek2()) {
                // komentarze // ...
                (Some('/'), Some('/')) => {
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        self.bump();
                    }
                }
                (Some(c), _) if c.is_whitespace() => { self.bump(); }
                _ => break,
            }
        }
    }

    fn read_number(&mut self) -> Result<TokenKind, LexError> {
        let (line, col) = (self.line, self.col);

        // hex: 0x10
        if self.peek() == Some('0') && matches!(self.peek2(), Some('x') | Some('X')) {
            self.bump();
            self.bump();
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() { s.push(c); self.bump(); } else { break; }
            }
            let v = u64::from_str_radix(&s, 16).map_err(|_| LexError {
                msg: format!("niepoprawny hex: 0x{}", s),
                line, col,
            })?;
            return Ok(TokenKind::Int(v));
        }

        // dziesietna
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { s.push(c); self.bump(); } else { break; }
        }
        let first: u64 = s.parse().map_err(|_| LexError {
            msg: format!("niepoprawna liczba: {}", s),
            line, col,
        })?;

        // layout NxM: 1x16  (ale NIE 0x - to hex!)
        if matches!(self.peek(), Some('x') | Some('X'))
            && self.peek2().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            self.bump(); // 'x'
            let mut s2 = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() { s2.push(c); self.bump(); } else { break; }
            }
            let second: u64 = s2.parse().map_err(|_| LexError {
                msg: format!("niepoprawny wymiar: {}x{}", first, s2),
                line, col,
            })?;
            return Ok(TokenKind::Dims(first, second));
        }

        Ok(TokenKind::Int(first))
    }

    fn read_ident(&mut self) -> TokenKind {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_cont(c) { s.push(c); self.bump(); } else { break; }
        }

        match s.as_str() {
            "Pub"    => TokenKind::Pub,
            "fn"     => TokenKind::Fn,
            "Const"  => TokenKind::Const,
            "Static" => TokenKind::Static,
            "Reg"    => TokenKind::Reg,
            "Mem"    => TokenKind::Mem,
            "if"     => TokenKind::If,
            "else"   => TokenKind::Else,
            "while"  => TokenKind::While,
            "return" => TokenKind::Return,
            "self"   => TokenKind::SelfKw,
            "u8"     => TokenKind::TypeU8,
            "u16"    => TokenKind::TypeU16,
            "u32"    => TokenKind::TypeU32,
            "u64"    => TokenKind::TypeU64,
            "i8"     => TokenKind::TypeI8,
            "i16"    => TokenKind::TypeI16,
            "i32"    => TokenKind::TypeI32,
            "i64"    => TokenKind::TypeI64,
            _        => TokenKind::Ident(s),
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_cont(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}