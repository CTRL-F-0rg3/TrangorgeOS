#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;

use vise_lexer::{Token, TokenKind};
use crate::ast::*;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Option<Module> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_top_level() {
                Some(item) => items.push(item),
                None => {
                    self.advance();
                }
            }
        }
        
        Some(Module { items })
    }

    fn parse_top_level(&mut self) -> Option<TopLevel> {
        match self.peek()?.kind {
            TokenKind::Fn => self.parse_function().map(TopLevel::Function),
            TokenKind::Struct => self.parse_struct().map(TopLevel::Struct),
            TokenKind::Enum => self.parse_enum().map(TopLevel::Enum),
            TokenKind::Static => self.parse_static().map(TopLevel::Static),
            _ => None,
        }
    }

    fn parse_function(&mut self) -> Option<Function> {
        self.expect(TokenKind::Fn)?;
        let name = self.expect_ident()?;
        
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            let qualifier = match self.peek()?.kind {
                TokenKind::In => { self.advance(); ParamQualifier::In }
                TokenKind::Out => { self.advance(); ParamQualifier::Out }
                TokenKind::InOut => { self.advance(); ParamQualifier::InOut }
                _ => ParamQualifier::In,
            };
            
            let ty = self.parse_type()?;
            let param_name = self.expect_ident()?;
            
            params.push(Param {
                name: param_name,
                ty,
                qualifier,
            });
            
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        self.expect(TokenKind::RParen)?;
        
        let return_type = if self.match_token(TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };
        
        let body = self.parse_block()?;
        
        Some(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_struct(&mut self) -> Option<StructDef> {
        self.expect(TokenKind::Struct)?;
        let name = self.expect_ident()?;
        
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let ty = self.parse_type()?;
            let field_name = self.expect_ident()?;
            fields.push((field_name, ty));
            
            self.expect(TokenKind::Semicolon)?;
        }
        
        self.expect(TokenKind::RBrace)?;
        
        Some(StructDef { name, fields })
    }

    fn parse_enum(&mut self) -> Option<EnumDef> {
        self.expect(TokenKind::Enum)?;
        let name = self.expect_ident()?;
        
        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let variant = self.expect_ident()?;
            variants.push(variant);
            
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        self.expect(TokenKind::RBrace)?;
        
        Some(EnumDef { name, variants })
    }

    fn parse_static(&mut self) -> Option<StaticData> {
        self.expect(TokenKind::Static)?;
        let name = self.expect_ident()?;
        
        self.expect(TokenKind::LBracket)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Colon)?;
        let count = self.expect_number()? as usize;
        self.expect(TokenKind::Semicolon)?;
        let init = self.expect_number()?;
        self.expect(TokenKind::RBracket)?;
        
        Some(StaticData { name, ty, count, init })
    }

    fn parse_block(&mut self) -> Option<Vec<Stmt>> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                self.advance();
            }
        }
        
        self.expect(TokenKind::RBrace)?;
        Some(stmts)
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek()?.kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::If => self.parse_if(),
            TokenKind::When => self.parse_when(),
            TokenKind::Return => {
                self.advance();
                let expr = if !self.check(TokenKind::Semicolon) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon)?;
                Some(Stmt::Return(expr))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::Semicolon)?;
                Some(Stmt::Expr(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Option<Stmt> {
        self.expect(TokenKind::Let)?;
        let ty = self.parse_type()?;
        let name = self.expect_ident()?;
        
        self.expect(TokenKind::Assign)?;
        let init = self.parse_expr()?;
        
        let neg_range = if self.match_token(TokenKind::Underscore) {
            self.expect(TokenKind::Arrow)?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Semicolon)?;
        
        Some(Stmt::Let { name, ty, init, neg_range })
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        self.expect(TokenKind::If)?;
        
        let cond = if !self.check(TokenKind::LBrace) {
            self.parse_expr()?
        } else {
            Expr::BoolLit(true)
        };
        
        let then_block = self.parse_block()?;
        
        let else_block = if self.match_token(TokenKind::Else) {
            if self.check(TokenKind::If) {
                Some(vec![self.parse_if()?])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        
        Some(Stmt::If { cond, then_block, else_block })
    }

    fn parse_when(&mut self) -> Option<Stmt> {
        self.expect(TokenKind::When)?;
        let target = self.expect_ident()?;
        
        self.expect(TokenKind::EqColon)?;
        self.expect(TokenKind::LBrace)?;
        
        let mut cases = Vec::new();
        
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let pattern = self.parse_expr()?;
            let body = self.parse_block()?;
            cases.push(WhenCase { pattern, body });
        }
        
        self.expect(TokenKind::RBrace)?;
        
        Some(Stmt::When { target, cases })
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let expr = self.parse_or()?;
        
        if self.match_token(TokenKind::Assign) {
            let rhs = self.parse_expr()?;
            return Some(Expr::Binary {
                op: BinOp::Eq,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            });
        }
        
        Some(expr)
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut left = self.parse_and()?;
        
        while self.match_token(TokenKind::Or) {
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(TokenKind::And) {
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinOp::And,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut left = self.parse_comparison()?;
        
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_addition()?;
        
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_addition(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplication()?;
        
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_multiplication(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let op = match self.peek()?.kind {
            TokenKind::Minus => UnaryOp::Neg,
            TokenKind::Not => UnaryOp::Not,
            TokenKind::Tilde => UnaryOp::BitNot,
            _ => return self.parse_postfix(),
        };
        
        self.advance();
        let operand = self.parse_unary()?;
        
        Some(Expr::Unary {
            op,
            operand: Box::new(operand),
        })
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(TokenKind::Dot) {
                let field = self.expect_ident()?;
                expr = Expr::Member {
                    object: Box::new(expr),
                    field,
                };
            } else if self.match_token(TokenKind::LBracket) {
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(TokenKind::LParen) {
                if let Expr::Ident(func) = expr {
                    let mut args = Vec::new();
                    
                    while !self.check(TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expr()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                    
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call { func, args };
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let token = self.peek()?;
        
        match token.kind {
            TokenKind::IntLiteral(val) => {
                self.advance();
                Some(Expr::IntLit(val))
            }
            TokenKind::FloatLiteral(val) => {
                self.advance();
                Some(Expr::FloatLit(val))
            }
            TokenKind::BoolLiteral(val) => {
                self.advance();
                Some(Expr::BoolLit(val))
            }
            TokenKind::Ident => {
                let name = self.expect_ident()?;
                Some(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Some(expr)
            }
            _ => None,
        }
    }

    fn parse_type(&mut self) -> Option<Type> {
        let token = self.peek()?;
        
        let ty = match token.kind {
            TokenKind::Float => { self.advance(); Type::Float }
            TokenKind::Int => { self.advance(); Type::Int }
            TokenKind::Uint => { self.advance(); Type::Uint }
            TokenKind::Bool => { self.advance(); Type::Bool }
            TokenKind::Vec2 => { self.advance(); Type::Vec2 }
            TokenKind::Vec3 => { self.advance(); Type::Vec3 }
            TokenKind::Vec4 => { self.advance(); Type::Vec4 }
            TokenKind::Mat3 => { self.advance(); Type::Mat3 }
            TokenKind::Mat4 => { self.advance(); Type::Mat4 }
            TokenKind::Sampler2D => { self.advance(); Type::Sampler2D }
            TokenKind::Image2D => { self.advance(); Type::Image2D }
            TokenKind::Ident => {
                let name = self.expect_ident()?;
                Type::Custom(name)
            }
            _ => return None,
        };
        
        if self.match_token(TokenKind::LBracket) {
            let size = self.expect_number()? as usize;
            self.expect(TokenKind::RBracket)?;
            Some(Type::Array(Box::new(ty), size))
        } else {
            Some(ty)
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1)
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |t| core::mem::discriminant(&t.kind) == core::mem::discriminant(&kind))
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<&Token> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }

    fn expect_ident(&mut self) -> Option<String> {
        if let Some(token) = self.peek() {
            if let TokenKind::Ident = token.kind {
                self.advance();
                return Some("ident".to_string());
            }
        }
        None
    }

    fn expect_number(&mut self) -> Option<i64> {
        if let Some(token) = self.peek() {
            if let TokenKind::IntLiteral(val) = token.kind {
                self.advance();
                return Some(val);
            }
        }
        None
    }

    fn is_at_end(&self) -> bool {
        self.peek().map_or(true, |t| t.kind == TokenKind::Eof)
    }
}