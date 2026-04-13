mod error;
mod expr;
mod func;
mod module;
mod stmt;
mod struct_;

use crate::ast::*;
use crate::error::Span;
use crate::parser::error::FinalParseResult;
use crate::tokens::{Assoc, Ident, Name, Op, ParamKind, Selff, Token, TokenKind};
use crate::{token_matches, token_matches_opt};
use error::{ParseError, ParseResult};
use indexmap::IndexMap;
use std::ops::Range;

pub trait Parse {
    type Output;
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self::Output>;
}

pub struct ParserCtx {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) source_id: String,
}

pub type Parser = ParserCtx;

impl ParserCtx {
    pub fn new(tokens: Vec<Token>, source_id: String) -> Self {
        Self {
            tokens,
            pos: 0,
            source_id,
        }
    }

    pub fn parse(&mut self) -> FinalParseResult<Program> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !self.is_eof() {
            let stmt = Stmt::parse(self);
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

    pub(crate) fn skip_to_recovery_point(&mut self) {
        while !self.is_eof() {
            match self.peek() {
                Some(tok)
                    if token_matches!(
                        tok,
                        TokenKind::Semicolon
                            | TokenKind::RightBrace
                            | TokenKind::Let
                            | TokenKind::Fn
                            | TokenKind::Extern
                            | TokenKind::Intrinsic
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

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        if let Some(tok) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub(crate) fn prev(&mut self) -> Option<Token> {
        if self.pos > 0 {
            self.pos -= 1;
            self.tokens.get(self.pos).cloned()
        } else {
            None
        }
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(crate) fn current_span(&self) -> Span {
        self.peek()
            .expect("current_span called but no current token exists")
            .span()
            .clone()
    }

    pub(crate) fn eof_span(&self) -> Span {
        self.tokens
            .last()
            .expect("eof_span called but no tokens exist")
            .span()
            .shrink_to_hi()
    }

    pub(crate) fn prev_span(&self) -> Span {
        assert!(
            self.pos > 0,
            "prev_span called but no previous token exists"
        );
        self.tokens[self.pos - 1].span().clone()
    }

    pub(crate) fn expect(&mut self, expected: &TokenKind) -> ParseResult<()> {
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(ref t) if t.kind() == expected => Ok(()),
            Some(t) => {
                let span = t.span().into_range();
                Err(ParseError::unexpected_token(
                    Some(t),
                    &format!("{expected:?}"),
                    span,
                    source_id,
                ))
            }
            None => {
                let span = self.eof_span().into_range();
                Err(ParseError::unexpected_token(
                    None,
                    &format!("{expected:?}"),
                    span,
                    source_id,
                ))
            }
        }
    }

    pub(crate) fn expect_identifier(&mut self) -> ParseResult<Ident> {
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(tok) => match tok.kind() {
                TokenKind::Ident(name) => Ok(name.clone()),
                _ => {
                    let span = tok.span().into_range();
                    Err(ParseError::unexpected_token(
                        Some(tok),
                        "identifier",
                        span,
                        source_id,
                    ))
                }
            },
            None => {
                let span = self.eof_span().into_range();
                Err(ParseError::unexpected_token(
                    None,
                    "identifier",
                    span,
                    source_id,
                ))
            }
        }
    }

    pub(crate) fn expect_keyword(&mut self, expected: &TokenKind) -> ParseResult<()> {
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(ref t) if t.kind() == expected => Ok(()),
            Some(t) => {
                let span = t.span().into_range();
                Err(ParseError::unexpected_token(
                    Some(t),
                    &format!("'{expected:?}'"),
                    span,
                    source_id,
                ))
            }
            None => {
                let span = self.eof_span().into_range();
                Err(ParseError::unexpected_token(
                    None,
                    &format!("'{expected:?}'"),
                    span,
                    source_id,
                ))
            }
        }
    }

    pub(crate) fn expect_type(&mut self) -> ParseResult<Types> {
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(tok) => match tok.kind() {
                TokenKind::LeftBracket => {
                    let size = self
                        .next()
                        .and_then(|t| match t.kind() {
                            TokenKind::Number(n) => Some(*n as usize),
                            _ => None,
                        })
                        .ok_or_else(|| {
                            let span = self.current_span().into_range();
                            ParseError::unexpected_token(
                                self.peek().cloned(),
                                "array size",
                                span,
                                source_id.clone(),
                            )
                        })?;
                    self.expect(&TokenKind::RightBracket)?;

                    Ok(Types::Array(self.expect_type()?.array_type(size)))
                }
                TokenKind::Ident(name) => Ok(Types::from_str(name)),
                TokenKind::Self_ => Ok(Types::Self_),
                TokenKind::BooleanAnd | TokenKind::And => {
                    let inner = self.expect_type()?;
                    Ok(Types::Pointer(Box::new(inner)))
                }
                _ => {
                    let span = tok.span().into_range();
                    Err(ParseError::unexpected_token(
                        Some(tok),
                        "type name",
                        span,
                        source_id,
                    ))
                }
            },
            None => {
                let span = self.eof_span().into_range();
                Err(ParseError::unexpected_token(
                    None,
                    "type name",
                    span,
                    source_id,
                ))
            }
        }
    }

    pub(crate) fn expect_string_literal(&mut self) -> ParseResult<String> {
        let source_id = self.source_id.clone();
        let token = self.next();
        match token {
            Some(tok) => match tok.kind() {
                TokenKind::String(s) => Ok(s.clone()),
                _ => {
                    let span = tok.span().into_range();
                    Err(ParseError::unexpected_token(
                        Some(tok),
                        "string literal",
                        span,
                        source_id,
                    ))
                }
            },
            None => {
                let span = self.eof_span().into_range();
                Err(ParseError::unexpected_token(
                    None,
                    "string literal",
                    span,
                    source_id,
                ))
            }
        }
    }

    pub(crate) fn consume_if(&mut self, expected: &TokenKind) -> bool {
        if self.peek().is_some_and(|tok| tok.kind() == expected) {
            self.next();
            true
        } else {
            false
        }
    }

    pub(crate) fn parse_comma_separated<T, F>(
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

    pub(crate) fn parse_comma_separated_with_variadic<T, F>(
        &mut self,
        mut parse_item: F,
        variadic: &mut bool,
    ) -> ParseResult<Vec<T>>
    where
        F: FnMut(&mut Self) -> ParseResult<T>,
    {
        let mut items = Vec::new();

        while !self
            .peek()
            .is_some_and(|tok| tok.kind() == &TokenKind::RightParen)
        {
            if self
                .peek()
                .is_some_and(|tok| tok.kind() == &TokenKind::Ellipsis)
            {
                self.next();
                *variadic = true;
                break;
            }
            items.push(parse_item(self)?);
            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(items)
    }
}
