mod error;

use crate::ast::*;
use crate::error::Span;
use crate::parser::error::FinalParseResult;
use crate::token_matches;
use crate::tokens::{Assoc, Ident, Op, Token, TokenKind};
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
            Ok(Program::new(stmts))
        } else {
            Err(error::FinalParseError::new(errors))
        }
    }

    fn skip_to_recovery_point(&mut self) {
        while !self.is_eof() {
            match self.peek() {
                Some(tok)
                    if token_matches!(
                        tok,
                        TokenKind::Semicolon
                            | TokenKind::RightBrace
                            | TokenKind::Let
                            | TokenKind::Fn
                            | TokenKind::Struct
                            | TokenKind::If
                            | TokenKind::While
                            | TokenKind::For
                            | TokenKind::Return
                    ) =>
                {
                    if token_matches!(tok, TokenKind::Semicolon) {
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
            match tok.kind() {
                TokenKind::Newline | TokenKind::Semicolon => {
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
        match token.kind() {
            TokenKind::LeftParen => 1,
            TokenKind::RightParen => 1,
            TokenKind::LeftBrace => 1,
            TokenKind::RightBrace => 1,
            TokenKind::LeftBracket => 1,
            TokenKind::RightBracket => 1,
            TokenKind::Comma => 1,
            TokenKind::Assign => 1,
            TokenKind::Dot => 1,
            TokenKind::Range => 2,
            TokenKind::Minus => 1,
            TokenKind::Plus => 1,
            TokenKind::Multiply => 1,
            TokenKind::Divide => 1,
            TokenKind::MinusEqual => 2,
            TokenKind::PlusEqual => 2,
            TokenKind::Colon => 1,
            TokenKind::Semicolon => 1,
            TokenKind::Bang => 1,
            TokenKind::BangEqual => 2,
            TokenKind::EqualEqual => 2,
            TokenKind::Greater => 1,
            TokenKind::GreaterEqual => 2,
            TokenKind::Less => 1,
            TokenKind::LessEqual => 2,
            TokenKind::Ident(ident) => ident.inner().len(),
            TokenKind::String(s) => s.len() + 2, // +2 for quotes
            TokenKind::Number(n) => n.to_string().len(),
            TokenKind::Float(f) => f.to_string().len(),
            TokenKind::And => 3,
            TokenKind::BoleanAnd => 2,
            TokenKind::Struct => 6,
            TokenKind::Else => 4,
            TokenKind::False => 5,
            TokenKind::For => 3,
            TokenKind::If => 2,
            TokenKind::Return => 6,
            TokenKind::Break => 5,
            TokenKind::Continue => 8,
            TokenKind::Or => 2,
            TokenKind::BooleanOr => 2,
            TokenKind::True => 4,
            TokenKind::While => 5,
            TokenKind::Fn => 2,
            TokenKind::Let => 3,
            TokenKind::Newline => 1,
        }
    }

    fn advance_char_pos(&mut self, token: &Token) {
        self.char_pos += self.estimate_token_length(token);
    }

    fn expect(&mut self, expected: &TokenKind) -> ParseResult<()> {
        let start_pos = self.char_pos;
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(ref t) if t.kind() == expected => Ok(()),
            other => {
                let span = other
                    .as_ref()
                    .map(|t| t.span().into_range())
                    .unwrap_or_else(|| start_pos..self.char_pos);
                Err(ParseError::unexpected_token(
                    other,
                    &format!("{expected:?}"),
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
            Some(tok) => match tok.kind() {
                TokenKind::Ident(name) => name.clone(),
                _ => {
                    let span = tok.span().into_range();
                    return Err(ParseError::unexpected_token(
                        Some(tok),
                        "identifier",
                        span,
                        source_id,
                    ));
                }
            },
            None => {
                let span = start_pos..self.char_pos;
                return Err(ParseError::unexpected_token(
                    None,
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
            Some(tok) => match tok.kind() {
                TokenKind::Ident(name) => Types::from_str(name),
                _ => {
                    let span = tok.span().into_range();
                    return Err(ParseError::unexpected_token(
                        Some(tok),
                        "type name",
                        span,
                        source_id,
                    ));
                }
            },
            None => {
                let span = start_pos..self.char_pos;
                return Err(ParseError::unexpected_token(
                    None,
                    "type name",
                    span,
                    source_id,
                ));
            }
        };
        Ok(result)
    }

    fn consume_if(&mut self, expected: &TokenKind) -> bool {
        if self.peek().is_some_and(|tok| tok.kind() == expected) {
            self.next();
            true
        } else {
            false
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(tok) if token_matches!(tok, TokenKind::Struct) => self.parse_struct_def(),
            Some(tok) if token_matches!(tok, TokenKind::Fn) => self.parse_function_def(),
            Some(tok) if token_matches!(tok, TokenKind::Let) => self.parse_let_stmt(),
            Some(tok) if token_matches!(tok, TokenKind::If) => self.parse_if_stmt(),
            Some(tok) if token_matches!(tok, TokenKind::While) => self.parse_while_stmt(),
            Some(tok) if token_matches!(tok, TokenKind::For) => self.parse_for_stmt(),
            Some(tok) if token_matches!(tok, TokenKind::Return) => self.parse_return_stmt(),
            Some(tok) if token_matches!(tok, TokenKind::Continue) => self.parse_continue_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_struct_def(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::Struct)?;
        let name = self.expect_identifier()?;
        let fields = self.parse_field_list()?;
        let end = self.char_pos;

        Ok(Stmt::StructDef(StructDef {
            name,
            fields,
            span: Span::new(start, end),
        }))
    }

    fn parse_function_def(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_identifier()?;
        let params = self.parse_parameter_list()?;
        let return_type = self.parse_optional_return_type()?;
        let body = self.parse_block()?;
        let end = self.char_pos;

        Ok(Stmt::FuncDef(FuncDef {
            name,
            params,
            return_type,
            body: Block::new(body.into(), Span::new(start, end)),
            span: Span::new(start, end),
        }))
    }

    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::Let)?;
        let name = self.expect_identifier()?;

        let var_type = if self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::Colon))
        {
            Some(self.parse_optional_custom_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_let_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        let end = self.char_pos;

        Ok(Stmt::Let(Let {
            name,
            declared_type: var_type,
            value: ExprAst {
                expr: value,
                span: Span::new(start, end),
            },
            span: Span::new(start, end),
        }))
    }

    fn parse_if_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::If)?;
        let cond = self.parse_if_expr()?;
        let then_branch = self.parse_stmt_or_block()?;
        let else_branch = if self.consume_if(&TokenKind::Else) {
            Some(self.parse_stmt_or_block()?)
        } else {
            None
        };
        let end = self.char_pos;
        let span = Span::new(start, end);

        Ok(Stmt::If(If {
            cond: ExprAst { expr: cond, span },
            then_branch: Block::new(then_branch.into(), span),
            else_branch: else_branch.map(|p| Block::new(p.into(), span)),
            span: Span::new(start, end),
        }))
    }

    fn parse_while_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::While)?;
        let cond = self.parse_while_expr()?;
        let body = self.parse_stmt_or_block()?;
        let end = self.char_pos;

        Ok(Stmt::While(While {
            cond,
            body: Block::new(body.into(), Span::new(start, end)),
            span: Span::new(start, end),
        }))
    }

    fn parse_for_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::For)?;
        let init = self.parse_stmt_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        let cond = self.parse_while_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        let update = self.parse_stmt_expr()?;
        let body = self.parse_stmt_or_block()?;
        let end = self.char_pos;
        let span = Span::new(start, end);

        Ok(Stmt::For(For {
            init: ExprAst { expr: init, span },
            cond: ExprAst { expr: cond, span },
            update: ExprAst { expr: update, span },
            body: Block::new(body.into(), Span::new(start, end)),
            span: Span::new(start, end),
        }))
    }

    fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::Return)?;
        let expr = if !self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::Semicolon))
        {
            Some(self.parse_stmt_expr()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon)?;
        let end = self.char_pos;
        let span = Span::new(start, end);
        let expr = expr.map(|e| ExprAst { expr: e, span });

        Ok(Stmt::Return(Return { expr, span }))
    }

    fn parse_continue_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        self.expect(&TokenKind::Continue)?;
        self.expect(&TokenKind::Semicolon)?;
        let end = self.char_pos;
        Ok(Stmt::Continue(Continue {
            span: Span::new(start, end),
        }))
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.char_pos;
        let expr = self.parse_stmt_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        let end = self.char_pos;
        Ok(Stmt::Expr(ExprAst {
            expr,
            span: Span::new(start, end),
        }))
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();

        while !self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
        {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(stmts)
    }

    fn parse_stmt_or_block(&mut self) -> ParseResult<Vec<Stmt>> {
        if self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::LeftBrace))
        {
            self.parse_block()
        } else {
            Ok(vec![self.parse_stmt()?])
        }
    }

    fn parse_parameter_list(&mut self) -> ParseResult<Vec<(Ident, Types)>> {
        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_comma_separated(
            |parser| {
                let name = parser.expect_identifier()?;
                parser.expect(&TokenKind::Colon)?;
                let param_type = parser.expect_type()?;
                Ok((name, param_type))
            },
            &TokenKind::RightParen,
        )?;
        self.expect(&TokenKind::RightParen)?;
        Ok(params)
    }

    fn parse_field_list(&mut self) -> ParseResult<IndexMap<Ident, Types>> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut fields = IndexMap::new();

        while !self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
        {
            let field_name = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let field_type = self.expect_type()?;
            fields.insert(field_name, field_type);

            if !self.consume_if(&TokenKind::Comma)
                && !self
                    .peek()
                    .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
            {
                break;
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(fields)
    }

    fn parse_optional_return_type(&mut self) -> ParseResult<Types> {
        if self.consume_if(&TokenKind::Colon) {
            self.expect_type()
        } else {
            Ok(Types::Void)
        }
    }

    fn parse_comma_separated<T, F>(
        &mut self,
        mut parse_item: F,
        terminator: &TokenKind,
    ) -> ParseResult<Vec<T>>
    where
        F: FnMut(&mut Self) -> ParseResult<T>,
    {
        let mut items = Vec::new();

        while !self.peek().is_some_and(|tok| tok.kind() == terminator) {
            items.push(parse_item(self)?);

            if !self.consume_if(&TokenKind::Comma) {
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
        if self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::Minus))
        {
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
            Some(tok) => match tok.kind() {
                TokenKind::Number(n) => Ok(Expr::Value(Value::Number(*n))),
                TokenKind::Float(f) => Ok(Expr::Value(Value::Float(*f))),
                TokenKind::String(s) => Ok(Expr::Value(Value::String(s.clone()))),
                TokenKind::True => Ok(Expr::Value(Value::Boolean(true))),
                TokenKind::False => Ok(Expr::Value(Value::Boolean(false))),
                TokenKind::Ident(name) => {
                    if allow_struct_literals {
                        self.parse_postfix_expr_with_struct(name.clone())
                    } else {
                        self.parse_postfix_expr(name.clone())
                    }
                }
                TokenKind::LeftParen => {
                    let expr = if allow_struct_literals {
                        self.parse_assignment()?
                    } else {
                        self.parse_binary_with_flags(0, false)?
                    };
                    self.expect(&TokenKind::RightParen)?;
                    Ok(expr)
                }
                _ => {
                    let span = tok.span().into_range();
                    Err(ParseError::unexpected_token(
                        Some(tok),
                        "expression",
                        span,
                        source_id,
                    ))
                }
            },
            None => {
                let span = start_pos..self.char_pos;
                Err(ParseError::unexpected_token(
                    None,
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
                    op,
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
            Some(token)
                if token_matches!(
                    token,
                    TokenKind::Assign | TokenKind::PlusEqual | TokenKind::MinusEqual
                ) =>
            {
                Some(token)
            }
            _ => None,
        }
    }

    fn parse_binary(&mut self, min_prec: u8) -> ParseResult<Expr> {
        self.parse_binary_with_flags(min_prec, true)
    }

    fn parse_postfix_expr(&mut self, name: Ident) -> ParseResult<Expr> {
        match self.peek() {
            Some(tok) if token_matches!(tok, TokenKind::LeftParen) => {
                self.parse_function_call(name)
            }
            _ => Ok(Expr::Ident(name)),
        }
    }

    fn parse_postfix_expr_with_struct(&mut self, name: Ident) -> ParseResult<Expr> {
        match self.peek() {
            Some(tok) if token_matches!(tok, TokenKind::LeftParen) => {
                self.parse_function_call(name)
            }
            Some(tok) if token_matches!(tok, TokenKind::LeftBrace) => {
                self.parse_struct_literal(name)
            }
            _ => Ok(Expr::Ident(name)),
        }
    }

    fn parse_struct_literal(&mut self, name: Ident) -> ParseResult<Expr> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut fields = IndexMap::new();

        while !self
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
        {
            let field_name = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let field_value = self.parse_assignment()?;
            fields.insert(field_name, field_value);

            if !self.consume_if(&TokenKind::Comma)
                && !self
                    .peek()
                    .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
            {
                break;
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(Expr::StructInit { name, fields })
    }

    fn parse_optional_custom_type(&mut self) -> ParseResult<Types> {
        self.expect(&TokenKind::Colon)?;

        let type_name = self.expect_identifier()?;
        Ok(Types::from_str(&type_name))
    }

    fn parse_function_call(&mut self, func_name: Ident) -> ParseResult<Expr> {
        self.expect(&TokenKind::LeftParen)?;
        let args =
            self.parse_comma_separated(|parser| parser.parse_assignment(), &TokenKind::RightParen)?;
        self.expect(&TokenKind::RightParen)?;

        Ok(Expr::Call {
            func: func_name,
            args,
        })
    }
}
