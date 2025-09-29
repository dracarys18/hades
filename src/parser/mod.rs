mod error;

use crate::ast::{Expr, Program, Stmt, Types};
use crate::parser::error::FinalParseResult;
use crate::tokens::{Assoc, Ident, Op, Token};
use error::{ParseError, ParseResult};
use indexmap::IndexMap;
use std::ops::Range;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    line: usize,
    col: usize,
    source_id: String,
    char_pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source_id: String) -> Self {
        Self {
            tokens,
            pos: 0,
            line: 1,
            col: 1,
            source_id,
            char_pos: 0,
        }
    }

    pub fn parse(&mut self) -> FinalParseResult<Program> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !self.is_eof() {
            let stmt = self.parse_stmt();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    errors.push(e);
                    self.skip_to_recovery_point();
                }
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            Err(error::FinalParseError::new(errors))
        }
    }

    fn skip_to_recovery_point(&mut self) {
        while !self.is_eof() {
            match self.peek() {
                Some(Token::Semicolon)
                | Some(Token::RightBrace)
                | Some(Token::Let)
                | Some(Token::Fn)
                | Some(Token::Struct)
                | Some(Token::If)
                | Some(Token::While)
                | Some(Token::For)
                | Some(Token::Return) => {
                    if matches!(self.peek(), Some(Token::Semicolon)) {
                        self.next();
                    }
                    break;
                }
                _ => {
                    self.next();
                }
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if let Some(tok) = self.tokens.get(self.pos).cloned() {
            self.advance_char_pos(&tok);
            match tok {
                Token::Newline | Token::Semicolon => {
                    self.line += 1;
                    self.col = 1;
                }
                _ => {
                    self.col += 1;
                }
            }
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn current_span(&self) -> Range<usize> {
        self.char_pos..self.char_pos + 1
    }

    fn estimate_token_length(&self, token: &Token) -> usize {
        match token {
            Token::LeftParen => 1,
            Token::RightParen => 1,
            Token::LeftBrace => 1,
            Token::RightBrace => 1,
            Token::LeftBracket => 1,
            Token::RightBracket => 1,
            Token::Comma => 1,
            Token::Assign => 1,
            Token::Dot => 1,
            Token::Range => 2,
            Token::Minus => 1,
            Token::Plus => 1,
            Token::Multiply => 1,
            Token::Divide => 1,
            Token::MinusEqual => 2,
            Token::PlusEqual => 2,
            Token::Colon => 1,
            Token::Semicolon => 1,
            Token::Bang => 1,
            Token::BangEqual => 2,
            Token::EqualEqual => 2,
            Token::Greater => 1,
            Token::GreaterEqual => 2,
            Token::Less => 1,
            Token::LessEqual => 2,
            Token::Ident(s) => s.len(),
            Token::String(s) => s.len() + 2, // +2 for quotes
            Token::Number(n) => n.to_string().len(),
            Token::Float(f) => f.to_string().len(),
            Token::And => 3,
            Token::BoleanAnd => 2,
            Token::Struct => 6,
            Token::Else => 4,
            Token::False => 5,
            Token::For => 3,
            Token::If => 2,
            Token::Return => 6,
            Token::Break => 5,
            Token::Continue => 8,
            Token::Or => 2,
            Token::BooleanOr => 2,
            Token::True => 4,
            Token::While => 5,
            Token::Fn => 2,
            Token::Let => 3,
            _ => 1,
        }
    }

    fn advance_char_pos(&mut self, token: &Token) {
        self.char_pos += self.estimate_token_length(token);
    }

    fn expect(&mut self, expected: &Token) -> ParseResult<()> {
        let start_pos = self.char_pos;
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(ref t) if t == expected => Ok(()),
            other => {
                let span = start_pos..self.char_pos;
                Err(ParseError::unexpected_token(
                    other,
                    &expected.to_string(),
                    span,
                    source_id,
                ))
            }
        }
    }

    fn expect_identifier(&mut self) -> ParseResult<Ident> {
        let start_pos = self.char_pos;
        let source_id = self.source_id.clone();
        let token = self.next();
        let result = match token {
            Some(Token::Ident(name)) => name,
            other => {
                let span = start_pos..self.char_pos;
                return Err(ParseError::unexpected_token(
                    other,
                    "identifier",
                    span,
                    source_id,
                ));
            }
        };
        Ok(result)
    }

    fn expect_type(&mut self) -> ParseResult<Types> {
        let start_pos = self.char_pos;
        let source_id = self.source_id.clone();
        let token = self.next();
        let result = match token {
            Some(Token::Ident(name)) => Types::from_str(&name),
            other => {
                let span = start_pos..self.char_pos;
                return Err(ParseError::unexpected_token(
                    other,
                    "type name",
                    span,
                    source_id,
                ));
            }
        };
        Ok(result)
    }

    fn consume_if(&mut self, token: &Token) -> bool {
        if self.peek() == Some(token) {
            self.next();
            true
        } else {
            false
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(Token::Struct) => self.parse_struct_def(),
            Some(Token::Fn) => self.parse_function_def(),
            Some(Token::Let) => self.parse_let_stmt(),
            Some(Token::If) => self.parse_if_stmt(),
            Some(Token::While) => self.parse_while_stmt(),
            Some(Token::For) => self.parse_for_stmt(),
            Some(Token::Return) => self.parse_return_stmt(),
            Some(Token::Continue) => self.parse_continue_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_struct_def(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Struct)?;
        let name = self.expect_identifier()?;
        let fields = self.parse_field_list()?;

        Ok(Stmt::StructDef { name, fields })
    }

    fn parse_function_def(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Fn)?;
        let name = self.expect_identifier()?;
        let params = self.parse_parameter_list()?;
        let return_type = self.parse_optional_return_type()?;
        let body = self.parse_block()?;

        Ok(Stmt::FuncDef {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Let)?;
        let name = self.expect_identifier()?;
        self.expect(&Token::Assign)?;
        let value = self.parse_let_expr()?;
        self.expect(&Token::Semicolon)?;

        Ok(Stmt::Let { name, value })
    }

    fn parse_if_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::If)?;
        let cond = self.parse_if_expr()?;
        let then_branch = self.parse_stmt_or_block()?;
        let else_branch = if self.consume_if(&Token::Else) {
            Some(self.parse_stmt_or_block()?)
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::While)?;
        let cond = self.parse_while_expr()?;
        let body = self.parse_stmt_or_block()?;

        Ok(Stmt::While { cond, body })
    }

    fn parse_for_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::For)?;
        let init = Box::new(self.parse_stmt()?);
        let cond = self.parse_while_expr()?;
        self.expect(&Token::Semicolon)?;
        let update = self.parse_stmt_expr()?;
        let body = self.parse_stmt_or_block()?;

        Ok(Stmt::For {
            init,
            cond,
            update,
            body,
        })
    }

    fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Return)?;
        let expr = if self.peek() != Some(&Token::Semicolon) {
            Some(self.parse_stmt_expr()?)
        } else {
            None
        };
        self.expect(&Token::Semicolon)?;

        Ok(Stmt::Return(expr))
    }

    fn parse_continue_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(&Token::Continue)?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Continue)
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.parse_stmt_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        self.expect(&Token::LeftBrace)?;
        let mut stmts = Vec::new();

        while self.peek() != Some(&Token::RightBrace) {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(&Token::RightBrace)?;
        Ok(stmts)
    }

    fn parse_stmt_or_block(&mut self) -> ParseResult<Vec<Stmt>> {
        if self.peek() == Some(&Token::LeftBrace) {
            self.parse_block()
        } else {
            Ok(vec![self.parse_stmt()?])
        }
    }

    fn parse_parameter_list(&mut self) -> ParseResult<Vec<(Ident, Types)>> {
        self.expect(&Token::LeftParen)?;
        let params = self.parse_comma_separated(
            |parser| {
                let name = parser.expect_identifier()?;
                parser.expect(&Token::Colon)?;
                let param_type = parser.expect_type()?;
                Ok((name, param_type))
            },
            &Token::RightParen,
        )?;
        self.expect(&Token::RightParen)?;
        Ok(params)
    }

    fn parse_field_list(&mut self) -> ParseResult<IndexMap<Ident, Types>> {
        self.expect(&Token::LeftBrace)?;
        let mut fields = IndexMap::new();

        while self.peek() != Some(&Token::RightBrace) {
            let field_name = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let field_type = self.expect_type()?;
            fields.insert(field_name, field_type);

            if !self.consume_if(&Token::Comma) && self.peek() != Some(&Token::RightBrace) {
                break;
            }
        }

        self.expect(&Token::RightBrace)?;
        Ok(fields)
    }

    fn parse_optional_return_type(&mut self) -> ParseResult<Types> {
        if self.consume_if(&Token::Colon) {
            self.expect_type()
        } else {
            Ok(Types::Void)
        }
    }

    fn parse_comma_separated<T, F>(
        &mut self,
        mut parse_item: F,
        terminator: &Token,
    ) -> ParseResult<Vec<T>>
    where
        F: FnMut(&mut Self) -> ParseResult<T>,
    {
        let mut items = Vec::new();

        while self.peek() != Some(terminator) {
            items.push(parse_item(self)?);

            if !self.consume_if(&Token::Comma) {
                break;
            }
        }

        Ok(items)
    }

    pub fn parse_if_expr(&mut self) -> ParseResult<Expr> {
        self.parse_binary_with_flags(0, false)
    }

    pub fn parse_while_expr(&mut self) -> ParseResult<Expr> {
        self.parse_binary_with_flags(0, false)
    }

    pub fn parse_let_expr(&mut self) -> ParseResult<Expr> {
        self.parse_assignment()
    }

    pub fn parse_stmt_expr(&mut self) -> ParseResult<Expr> {
        self.parse_assignment()
    }

    fn parse_binary_with_flags(
        &mut self,
        min_prec: u8,
        allow_struct_literals: bool,
    ) -> ParseResult<Expr> {
        let mut left = self.parse_unary_with_flags(allow_struct_literals)?;

        while let Some(token) = self.peek() {
            let op = match Op::from_token(token) {
                Some(op) => op,
                None => break,
            };

            let prec_info = match op.get_precedence() {
                Some(p) => p,
                None => break,
            };

            if prec_info.prec < min_prec {
                break;
            }

            self.next();
            let next_min_prec = if prec_info.assoc == Assoc::Left {
                prec_info.prec + 1
            } else {
                prec_info.prec
            };

            let right = self.parse_binary_with_flags(next_min_prec, allow_struct_literals)?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary_with_flags(&mut self, allow_struct_literals: bool) -> ParseResult<Expr> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.next();
            let expr = self.parse_unary_with_flags(allow_struct_literals)?;
            Ok(Expr::Unary {
                op: Op::Minus,
                expr: Box::new(expr),
            })
        } else {
            self.parse_primary_with_flags(allow_struct_literals)
        }
    }

    fn parse_primary_with_flags(&mut self, allow_struct_literals: bool) -> ParseResult<Expr> {
        let start_pos = self.char_pos;
        let source_id = self.source_id.clone();

        let token = self.next();
        match token {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Float(f)) => Ok(Expr::Float(f)),
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::True) => Ok(Expr::Boolean(true)),
            Some(Token::False) => Ok(Expr::Boolean(false)),
            Some(Token::Ident(name)) => {
                if allow_struct_literals {
                    self.parse_postfix_expr_with_struct(name)
                } else {
                    self.parse_postfix_expr(name)
                }
            }
            Some(Token::LeftParen) => {
                let expr = if allow_struct_literals {
                    self.parse_assignment()?
                } else {
                    self.parse_binary_with_flags(0, false)?
                };
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }
            other => {
                let span = start_pos..self.char_pos;
                Err(ParseError::unexpected_token(
                    other,
                    "expression",
                    span,
                    source_id,
                ))
            }
        }
    }

    fn parse_assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_binary(0)?;

        if self.peek_assignment_op().is_some() {
            if let Expr::Ident(name) = expr {
                let op = Op::from_token(&self.next().unwrap()).unwrap();
                let value = self.parse_assignment()?;
                return Ok(Expr::Assign {
                    name,
                    op: Some(op),
                    value: Box::new(value),
                });
            } else {
                let span = self.current_span();
                let source_id = self.source_id.clone();
                return Err(ParseError::invalid_assignment_target(span, source_id));
            }
        }

        Ok(expr)
    }

    fn peek_assignment_op(&self) -> Option<&Token> {
        match self.peek() {
            Some(token @ (Token::Assign | Token::PlusEqual | Token::MinusEqual)) => Some(token),
            _ => None,
        }
    }

    fn parse_binary(&mut self, min_prec: u8) -> ParseResult<Expr> {
        self.parse_binary_with_flags(min_prec, true)
    }

    fn parse_postfix_expr(&mut self, name: Ident) -> ParseResult<Expr> {
        match self.peek() {
            Some(Token::LeftParen) => self.parse_function_call(name),
            _ => Ok(Expr::Ident(name)),
        }
    }

    fn parse_postfix_expr_with_struct(&mut self, name: Ident) -> ParseResult<Expr> {
        match self.peek() {
            Some(Token::LeftParen) => self.parse_function_call(name),
            Some(Token::LeftBrace) => self.parse_struct_literal(name),
            _ => Ok(Expr::Ident(name)),
        }
    }

    fn parse_struct_literal(&mut self, struct_name: Ident) -> ParseResult<Expr> {
        self.expect(&Token::LeftBrace)?;
        let mut fields = IndexMap::new();

        while self.peek() != Some(&Token::RightBrace) {
            let field_name = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let field_value = self.parse_assignment()?;
            fields.insert(field_name, field_value);
            if !self.consume_if(&Token::Comma) && self.peek() != Some(&Token::RightBrace) {
                break;
            }
        }

        self.expect(&Token::RightBrace)?;
        Ok(Expr::StructInit {
            name: struct_name,
            fields,
        })
    }

    fn parse_function_call(&mut self, func_name: Ident) -> ParseResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let args =
            self.parse_comma_separated(|parser| parser.parse_assignment(), &Token::RightParen)?;
        self.expect(&Token::RightParen)?;

        Ok(Expr::Call {
            func: func_name,
            args,
        })
    }
}
