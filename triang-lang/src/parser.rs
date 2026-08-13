use crate::ast::*;
use crate::token::{Token, TokenKind};

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn kind(&self) -> TokenKind {
        self.cur().kind.clone()
    }

    fn at(&self, kind: &TokenKind) -> bool {
        &self.kind() == kind
    }

    fn bump(&mut self) -> TokenKind {
        let k = self.kind();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        k
    }

    fn err(&self, msg: String) -> ParseError {
        ParseError {
            msg,
            line: self.cur().line,
            col: self.cur().col,
        }
    }

    fn expect(&mut self, want: TokenKind) -> Result<TokenKind, ParseError> {
        if self.at(&want) {
            Ok(self.bump())
        } else {
            Err(self.err(format!("oczekiwano {:?}, otrzymano {:?}", want, self.kind())))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            TokenKind::Ident(s) => Ok(s),
            other => Err(self.err(format!("oczekiwano identyfikatora, otrzymano {:?}", other))),
        }
    }

    fn expect_int(&mut self) -> Result<u64, ParseError> {
        match self.bump() {
            TokenKind::Int(v) => Ok(v),
            other => Err(self.err(format!("oczekiwano liczby, otrzymano {:?}", other))),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();
        while !self.at(&TokenKind::Eof) {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let is_pub = self.at(&TokenKind::Pub);
        if is_pub {
            self.bump();
        }

        self.expect(TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;

        let mut params = Vec::new();
        while !self.at(&TokenKind::RParen) {
            let p = match self.kind() {
                TokenKind::SelfKw => {
                    self.bump();
                    Param::SelfParam
                }
                _ => Param::Typed(self.parse_type()?),
            };
            params.push(p);
            if self.at(&TokenKind::Semicolon) {
                self.bump();
            }
        }
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::LBrace)?;

        let body = self.parse_block()?;

        Ok(Function { is_pub, name, params, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.kind() {
            TokenKind::Const => self.parse_const(),
            TokenKind::Static => self.parse_static(),
            TokenKind::Reg => self.parse_reg(),
            TokenKind::Mem => self.parse_mem(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Ident(_) => self.parse_op_stmt(),
            other => Err(self.err(format!("nieoczekiwany token {:?}", other))),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.bump() {
            TokenKind::TypeU8 => Ok(Type::U8),
            TokenKind::TypeU16 => Ok(Type::U16),
            TokenKind::TypeU32 => Ok(Type::U32),
            TokenKind::TypeU64 => Ok(Type::U64),
            TokenKind::TypeI8 => Ok(Type::I8),
            TokenKind::TypeI16 => Ok(Type::I16),
            TokenKind::TypeI32 => Ok(Type::I32),
            TokenKind::TypeI64 => Ok(Type::I64),
            other => Err(self.err(format!("oczekiwano typu, otrzymano {:?}", other))),
        }
    }

    fn parse_layout(&mut self) -> Result<Layout, ParseError> {
        self.expect(TokenKind::LBracket)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Comma)?;
        let count = self.expect_int()?;
        self.expect(TokenKind::ShiftLeft)?;
        let shift = self.expect_int()?;
        self.expect(TokenKind::Semicolon)?;
        self.expect(TokenKind::RBracket)?;
        Ok(Layout { ty, count, shift })
    }

    fn parse_const(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        let from = self.parse_layout()?;
        self.expect(TokenKind::FatArrow)?;
        let to = self.parse_layout()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::ConstDecl { from, to })
    }

    fn parse_static(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        self.expect(TokenKind::LBracket)?;
        let dims = match self.bump() {
            TokenKind::Dims(a, b) => (a, b),
            other => return Err(self.err(format!("oczekiwano wymiarów NxM, otrzymano {:?}", other))),
        };
        self.expect(TokenKind::Semicolon)?;
        let format = self.expect_ident()?;
        self.expect(TokenKind::Arrow)?;
        let map_to = self.expect_int()?;
        self.expect(TokenKind::Semicolon)?;
        self.expect(TokenKind::RBracket)?;
        let op = self.parse_op_suffix(Target::Region)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::StaticDecl { dims, format, map_to, op })
    }

    fn parse_reg(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        let ty = self.parse_type()?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::RegDecl { ty, name })
    }

    fn parse_mem(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        self.expect(TokenKind::LBracket)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Comma)?;
        let len = self.expect_int()?;
        self.expect(TokenKind::RBracket)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::MemDecl { ty, len, name })
    }

    fn parse_target(&mut self) -> Result<Target, ParseError> {
        let name = self.expect_ident()?;
        if self.at(&TokenKind::LBracket) {
            self.bump();
            let idx = self.expect_int()?;
            self.expect(TokenKind::RBracket)?;
            Ok(Target::Indexed(name, idx))
        } else {
            Ok(Target::Named(name))
        }
    }

    fn parse_op_call(&mut self) -> Result<OpCall, ParseError> {
        let target = self.parse_target()?;
        self.parse_op_suffix(target)
    }

    fn parse_op_suffix(&mut self, target: Target) -> Result<OpCall, ParseError> {
        self.expect(TokenKind::ColonColon)?;
        let op = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        while !self.at(&TokenKind::RParen) {
            args.push(self.parse_expr()?);
            if self.at(&TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(OpCall { target, op, args })
    }

    fn parse_op_stmt(&mut self) -> Result<Stmt, ParseError> {
        let call = self.parse_op_call()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Op(call))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.kind() {
            TokenKind::Int(_) => match self.bump() {
                TokenKind::Int(v) => Ok(Expr::Int(v)),
                _ => unreachable!(),
            },
            TokenKind::Ident(_) => {
                let name = self.expect_ident()?;
                if self.at(&TokenKind::LBracket) {
                    self.bump();
                    let idx = self.expect_int()?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(Expr::Indexed(name, idx))
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            other => Err(self.err(format!("oczekiwano wyrażenia, otrzymano {:?}", other))),
        }
    }

    fn parse_cond(&mut self) -> Result<Cond, ParseError> {
        let lhs = self.parse_expr()?;
        let op = match self.bump() {
            TokenKind::EqEq => CmpOp::Eq,
            TokenKind::NotEq => CmpOp::NotEq,
            other => return Err(self.err(format!("oczekiwano == lub !=, otrzymano {:?}", other))),
        };
        let rhs = self.parse_expr()?;
        Ok(Cond { lhs, op, rhs })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        let cond = self.parse_cond()?;
        self.expect(TokenKind::LBrace)?;
        let then_body = self.parse_block()?;
        let mut else_body = Vec::new();
        if self.at(&TokenKind::Else) {
            self.bump();
            self.expect(TokenKind::LBrace)?;
            else_body = self.parse_block()?;
        }
        if self.at(&TokenKind::Semicolon) {
            self.bump();
        }
        Ok(Stmt::If { cond, then_body, else_body })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        let cond = self.parse_cond()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        if self.at(&TokenKind::Semicolon) {
            self.bump();
        }
        Ok(Stmt::While { cond, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        let expr = if self.at(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }
}