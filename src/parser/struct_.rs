use crate::ast::*;
use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::parser::error::ParseResult;
use crate::parser::expr::parse_assignment;
use crate::parser::func::FuncDef;
use crate::token_matches;
use crate::tokens::Name;
use crate::tokens::{Ident, ParamKind, TokenKind};
use indexmap::IndexMap;

pub(super) struct StructDef;

impl Parse for StructDef {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Struct)?;
        let ident = ctx.expect_identifier()?;
        let name = Name::new(ident.to_string(), ident.span().clone());
        let fields = parse_field_list(ctx, name.clone())?;
        let end = ctx.prev_span();

        Ok(Stmt::StructDef(crate::ast::StructDef {
            name,
            fields,
            span: start_tok.to(end),
        }))
    }
}

pub(super) fn parse_struct_literal(ctx: &mut ParserCtx, path: Vec<Ident>) -> ParseResult<Expr> {
    ctx.expect(&TokenKind::LeftBrace)?;
    let mut fields = IndexMap::new();

    while !ctx
        .peek()
        .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
    {
        let field_name = ctx.expect_identifier()?;
        ctx.expect(&TokenKind::Colon)?;
        let field_value = parse_assignment(ctx)?;
        fields.insert(field_name, field_value);

        if !ctx.consume_if(&TokenKind::Comma)
            && !ctx
                .peek()
                .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
        {
            break;
        }
    }

    ctx.expect(&TokenKind::RightBrace)?;
    Ok(Expr::StructInit(StructInitExpr { path, fields }))
}

pub(super) fn parse_field_list(
    ctx: &mut ParserCtx,
    struct_name: crate::tokens::Name,
) -> ParseResult<IndexMap<crate::tokens::Ident, FieldKind>> {
    ctx.expect(&TokenKind::LeftBrace)?;
    let mut fields = IndexMap::new();

    while !ctx
        .peek()
        .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
    {
        let field = ctx.peek().ok_or_else(|| {
            let span = ctx.eof_span().into_range();
            crate::parser::error::ParseError::unexpected_token(
                None,
                "field declaration",
                span,
                ctx.source_id.clone(),
            )
        })?;

        match field.kind() {
            TokenKind::Fn => {
                let mut func = FuncDef::parse(ctx)?.unwrap_func_def();
                let kind = func
                    .params
                    .iter()
                    .find(|(k, _)| matches!(k, ParamKind::Self_(_)))
                    .map_or(ReceiverKind::Value, |(_, t)| {
                        if matches!(t, Types::Pointer(_)) {
                            ReceiverKind::Pointer
                        } else {
                            ReceiverKind::Value
                        }
                    });
                func.receiver = Some(Receiver {
                    struct_name: struct_name.clone(),
                    kind,
                });
                let key = func.name.to_ident();
                fields.insert(key, FieldKind::Func(Box::new(func)));
            }
            TokenKind::Ident(field_name) => {
                let field_name = field_name.clone();
                ctx.next();
                ctx.expect(&TokenKind::Colon)?;
                let field_type = ctx.expect_type()?;
                fields.insert(field_name, FieldKind::Var(field_type));

                if !ctx.consume_if(&TokenKind::Comma)
                    && !ctx
                        .peek()
                        .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
                {
                    break;
                }
            }
            _ => Err(crate::parser::error::ParseError::unexpected_token(
                Some(field.clone()),
                "field declaration",
                field.span().into_range(),
                ctx.source_id.clone(),
            ))?,
        }
    }

    ctx.expect(&TokenKind::RightBrace)?;
    Ok(fields)
}
