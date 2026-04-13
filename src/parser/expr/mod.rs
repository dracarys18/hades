mod array;

use crate::ast::*;
use crate::parser::error::ParseResult;
use crate::parser::struct_::parse_struct_literal;
use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::tokens::{Assoc, Ident, Name, Op, TokenKind};
use crate::{token_matches, token_matches_opt};
use array::{parse_array_index, ArrayLiteral};

impl Parse for Expr {
    type Output = Expr;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Expr> {
        parse_assignment(ctx)
    }
}

pub(crate) fn parse_assignment(ctx: &mut ParserCtx) -> ParseResult<Expr> {
    let expr = parse_binary(ctx, 0)?;

    if peek_assignment_op(ctx).is_some() {
        match expr {
            Expr::Ident(name) => {
                let op = Op::from_token(&ctx.next().unwrap()).unwrap();
                let value = parse_assignment(ctx)?;
                return Ok(Expr::Assign(AssignExpr {
                    target: AssignTarget::Ident(name),
                    op,
                    value: Box::new(value),
                }));
            }
            Expr::FieldAccess(field) => {
                let op = Op::from_token(&ctx.next().unwrap()).unwrap();
                let value = parse_assignment(ctx)?;
                return Ok(Expr::Assign(AssignExpr {
                    target: AssignTarget::FieldAccess(field),
                    op,
                    value: Box::new(value),
                }));
            }
            Expr::ArrayIndex(index) => {
                let op = Op::from_token(&ctx.next().unwrap()).unwrap();
                let value = parse_assignment(ctx)?;
                return Ok(Expr::Assign(AssignExpr {
                    target: AssignTarget::ArrayIndex(index),
                    op,
                    value: Box::new(value),
                }));
            }
            Expr::Unary {
                op: Op::Deref,
                expr: inner,
            } => {
                let op = Op::from_token(&ctx.next().unwrap()).unwrap();
                let value = parse_assignment(ctx)?;
                return Ok(Expr::Assign(AssignExpr {
                    target: AssignTarget::Deref(inner),
                    op,
                    value: Box::new(value),
                }));
            }
            _ => {
                let span = ctx.current_span().into_range();
                let source_id = ctx.source_id.clone();
                return Err(crate::parser::error::ParseError::invalid_assignment_target(
                    span, source_id,
                ));
            }
        }
    }

    Ok(expr)
}

pub(crate) fn parse_binary_with_flags(
    ctx: &mut ParserCtx,
    min_prec: u8,
    allow_struct_literals: bool,
) -> ParseResult<Expr> {
    let mut left = parse_unary_with_flags(ctx, allow_struct_literals)?;

    while let Some(token) = ctx.peek() {
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

        ctx.next();
        let next_min_prec = if prec_info.assoc == Assoc::Left {
            prec_info.prec + 1
        } else {
            prec_info.prec
        };

        let right = parse_binary_with_flags(ctx, next_min_prec, allow_struct_literals)?;

        left = Expr::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        });
    }

    if ctx
        .peek()
        .is_some_and(|tok| token_matches!(tok, TokenKind::As))
    {
        ctx.next();
        let target_type = ctx.expect_type()?;
        left = Expr::As(AsExpression {
            expr: Box::new(left),
            target_type,
        });
    }

    Ok(left)
}

fn parse_binary(ctx: &mut ParserCtx, min_prec: u8) -> ParseResult<Expr> {
    parse_binary_with_flags(ctx, min_prec, true)
}

fn parse_unary_with_flags(ctx: &mut ParserCtx, allow_struct_literals: bool) -> ParseResult<Expr> {
    match ctx.peek() {
        tok if tok.is_some_and(|token| token_matches!(token, TokenKind::Minus)) => {
            ctx.next();
            let expr = parse_unary_with_flags(ctx, allow_struct_literals)?;
            Ok(Expr::Unary {
                op: Op::Minus,
                expr: Box::new(expr),
            })
        }
        tok if tok.is_some_and(|token| token_matches!(token, TokenKind::Bang)) => {
            ctx.next();
            let expr = parse_unary_with_flags(ctx, allow_struct_literals)?;
            Ok(Expr::Unary {
                op: Op::Not,
                expr: Box::new(expr),
            })
        }
        tok if tok.is_some_and(|token| token_matches!(token, TokenKind::BooleanAnd)) => {
            ctx.next();
            let expr = parse_unary_with_flags(ctx, allow_struct_literals)?;
            Ok(Expr::Unary {
                op: Op::Ref,
                expr: Box::new(expr),
            })
        }
        tok if tok.is_some_and(|token| token_matches!(token, TokenKind::Multiply)) => {
            ctx.next();
            let expr = parse_unary_with_flags(ctx, allow_struct_literals)?;
            let deref = Expr::Unary {
                op: Op::Deref,
                expr: Box::new(expr),
            };
            parse_postfix_chain(ctx, deref, allow_struct_literals)
        }
        _ => parse_primary_with_flags(ctx, allow_struct_literals),
    }
}

fn parse_primary_with_flags(ctx: &mut ParserCtx, allow_struct_literals: bool) -> ParseResult<Expr> {
    let source_id = ctx.source_id.clone();

    let token = ctx.next();
    match token {
        Some(tok) => match tok.kind() {
            TokenKind::Number(n) => Ok(Expr::Value(Value::Number(*n))),
            TokenKind::Float(f) => Ok(Expr::Value(Value::Float(*f))),
            TokenKind::String(s) => Ok(Expr::Value(Value::String(s.clone()))),
            TokenKind::Char(c) => Ok(Expr::Value(Value::Char(*c))),
            TokenKind::True => Ok(Expr::Value(Value::Boolean(true))),
            TokenKind::False => Ok(Expr::Value(Value::Boolean(false))),
            TokenKind::Null => Ok(Expr::Null),
            TokenKind::Ident(name) => {
                parse_postfix_chain(ctx, Expr::Ident(name.clone()), allow_struct_literals)
            }
            TokenKind::LeftParen => {
                let expr = if allow_struct_literals {
                    parse_assignment(ctx)?
                } else {
                    parse_binary_with_flags(ctx, 0, false)?
                };
                ctx.expect(&TokenKind::RightParen)?;
                parse_postfix_chain(ctx, expr, allow_struct_literals)
            }
            TokenKind::LeftBracket => ArrayLiteral::parse(ctx),
            TokenKind::Self_ => {
                let self_ident = crate::tokens::Ident::new("self".to_string(), tok.span().clone());
                parse_postfix_chain(ctx, Expr::Ident(self_ident), allow_struct_literals)
            }
            _ => {
                let span = tok.span().into_range();
                Err(crate::parser::error::ParseError::unexpected_token(
                    Some(tok),
                    "expression",
                    span,
                    source_id,
                ))
            }
        },
        None => {
            let span = ctx.current_span().into_range();
            Err(crate::parser::error::ParseError::unexpected_token(
                None,
                "expression",
                span,
                source_id,
            ))
        }
    }
}

fn parse_postfix_chain(
    ctx: &mut ParserCtx,
    mut expr: Expr,
    allow_struct_literals: bool,
) -> ParseResult<Expr> {
    loop {
        match ctx.peek() {
            Some(tok) if token_matches!(tok, TokenKind::Dot) => {
                ctx.next();
                let field_name = ctx.expect_identifier()?;
                if ctx
                    .peek()
                    .is_some_and(|tok| token_matches!(tok, TokenKind::LeftParen))
                {
                    ctx.next();
                    let args =
                        ctx.parse_comma_separated(|c| parse_assignment(c), &TokenKind::RightParen)?;
                    ctx.expect(&TokenKind::RightParen)?;
                    expr = Expr::Call(CallKind::Method(MethodCall {
                        receiver: Box::new(expr),
                        func: Name::new(field_name.inner().to_string(), field_name.span().clone()),
                        args,
                    }));
                } else {
                    expr = Expr::FieldAccess(FieldAccessExpr {
                        expr: Box::new(expr),
                        field: field_name,
                    });
                }
            }
            Some(tok) if token_matches!(tok, TokenKind::DoubleColon) => {
                let first = match expr {
                    Expr::Ident(ref name) => name.clone(),
                    _ => {
                        let span = ctx.current_span().into_range();
                        let source_id = ctx.source_id.clone();
                        return Err(crate::parser::error::ParseError::unexpected_token(
                            None,
                            "identifier before '::'",
                            span,
                            source_id,
                        ));
                    }
                };
                ctx.next(); // consume ::
                let mut path = vec![first];
                loop {
                    let segment = ctx.expect_identifier()?;
                    if ctx
                        .peek()
                        .is_some_and(|t| token_matches!(t, TokenKind::DoubleColon))
                    {
                        ctx.next(); // consume ::
                        path.push(segment);
                    } else if ctx
                        .peek()
                        .is_some_and(|t| token_matches!(t, TokenKind::LeftParen))
                    {
                        ctx.next(); // consume (
                        let args = ctx.parse_comma_separated(
                            |c| parse_assignment(c),
                            &TokenKind::RightParen,
                        )?;
                        ctx.expect(&TokenKind::RightParen)?;
                        expr = Expr::Call(CallKind::Qualified(QualifiedCall {
                            path,
                            func: Name::new(segment.inner().to_string(), segment.span().clone()),
                            args,
                        }));
                        break;
                    } else if allow_struct_literals
                        && ctx
                            .peek()
                            .is_some_and(|t| token_matches!(t, TokenKind::LeftBrace))
                    {
                        path.push(segment);
                        expr = parse_struct_literal(ctx, path)?;
                        break;
                    } else {
                        path.push(segment);
                        expr = Expr::Ident(path.last().unwrap().clone());
                        break;
                    }
                }
            }
            Some(tok) if token_matches!(tok, TokenKind::LeftBracket) => {
                ctx.next();
                let index_expr = parse_assignment(ctx)?;
                ctx.expect(&TokenKind::RightBracket)?;
                expr = Expr::ArrayIndex(ArrayIndexExpr {
                    expr: Box::new(expr),
                    index: Box::new(index_expr),
                });
            }
            Some(tok) if token_matches!(tok, TokenKind::LeftParen) => {
                if let Expr::Ident(func_name) = expr {
                    ctx.next();
                    let args =
                        ctx.parse_comma_separated(|c| parse_assignment(c), &TokenKind::RightParen)?;
                    ctx.expect(&TokenKind::RightParen)?;
                    expr = Expr::Call(CallKind::Function(FunctionCall {
                        func: Name::new(func_name.inner().to_string(), func_name.span().clone()),
                        args,
                    }));
                } else {
                    break;
                }
            }
            Some(tok) if allow_struct_literals && token_matches!(tok, TokenKind::LeftBrace) => {
                if let Expr::Ident(name) = expr {
                    expr = parse_struct_literal(ctx, vec![name])?;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn peek_assignment_op(ctx: &ParserCtx) -> Option<&crate::tokens::Token> {
    match ctx.peek() {
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
