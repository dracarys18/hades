use crate::ast::*;
use crate::parser::error::ParseResult;
use crate::parser::expr::parse_assignment;
use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::token_matches;
use crate::tokens::{Ident, TokenKind};

pub(super) struct ArrayLiteral;

impl Parse for ArrayLiteral {
    type Output = Expr;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Expr> {
        if ctx
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::RightBracket))
        {
            ctx.expect(&TokenKind::RightBracket)?;
            return Ok(Expr::Value(Value::Array(crate::ast::ArrayLiteral::new(
                vec![],
            ))));
        }

        let first = parse_assignment(ctx)?;

        if ctx.consume_if(&TokenKind::Semicolon) {
            let count_token = ctx.next();
            let count = match count_token.as_ref().map(|t| t.kind()) {
                Some(TokenKind::Number(n)) => *n as usize,
                _ => {
                    let span = ctx.current_span().into_range();
                    let source_id = ctx.source_id.clone();
                    return Err(crate::parser::error::ParseError::unexpected_token(
                        count_token,
                        "array repeat count",
                        span,
                        source_id,
                    ));
                }
            };
            ctx.expect(&TokenKind::RightBracket)?;
            return Ok(Expr::Value(Value::Array(
                crate::ast::ArrayLiteral::new_fill(first, count),
            )));
        }

        let mut elem = vec![first];
        if ctx.consume_if(&TokenKind::Comma) {
            let rest =
                ctx.parse_comma_separated(|c| parse_assignment(c), &TokenKind::RightBracket)?;
            elem.extend(rest);
        }
        ctx.expect(&TokenKind::RightBracket)?;
        Ok(Expr::Value(Value::Array(crate::ast::ArrayLiteral::new(
            elem,
        ))))
    }
}

pub(super) fn parse_array_index(ctx: &mut ParserCtx, name: Ident) -> ParseResult<Expr> {
    ctx.expect(&TokenKind::LeftBracket)?;
    let index_expr = parse_assignment(ctx)?;
    ctx.expect(&TokenKind::RightBracket)?;

    Ok(Expr::ArrayIndex(ArrayIndexExpr {
        expr: Box::new(Expr::Ident(name)),
        index: Box::new(index_expr),
    }))
}
