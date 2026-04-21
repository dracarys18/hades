use crate::ast::*;
use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::parser::error::ParseResult;
use crate::parser::stmt::parse_block;
use crate::tokens::{Name, ParamKind, Selff, TokenKind};

pub(super) struct FuncDef;

impl Parse for FuncDef {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Fn)?;
        let name_ident = ctx.expect_identifier()?;
        let name = Name::new(name_ident.inner().to_string(), name_ident.span().clone());
        let params = parse_parameter_list(ctx)?;
        let return_type = parse_optional_return_type(ctx)?;
        let body = parse_block(ctx)?;
        let end = ctx.prev_span();
        let span = start_tok.to(end);

        Ok(Stmt::FuncDef(crate::ast::FuncDef {
            name,
            receiver: None,
            params,
            return_type,
            body: FuncBody::Block(Block::new(body.into(), span.clone())),
            span,
        }))
    }
}

pub(super) fn parse_extern_fn(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
    let start_tok = ctx.current_span();
    ctx.expect(&TokenKind::Extern)?;
    ctx.expect(&TokenKind::Fn)?;
    let name_ident = ctx.expect_identifier()?;
    let name = Name::new(name_ident.inner().to_string(), name_ident.span().clone());
    let (params, variadic) = parse_parameter_list_with_variadic(ctx)?;
    let return_type = parse_optional_return_type(ctx)?;
    ctx.expect(&TokenKind::Semicolon)?;
    let end = ctx.prev_span();
    let span = start_tok.to(end);

    Ok(Stmt::FuncDef(crate::ast::FuncDef {
        name,
        receiver: None,
        params,
        return_type,
        body: FuncBody::Extern { variadic },
        span,
    }))
}

pub(super) fn parse_intrinsic_fn(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
    let start_tok = ctx.current_span();
    ctx.expect(&TokenKind::Intrinsic)?;
    ctx.expect(&TokenKind::Fn)?;
    let name_ident = ctx.expect_identifier()?;
    let name = Name::new(name_ident.inner().to_string(), name_ident.span().clone());
    let params = parse_parameter_list(ctx)?;
    let return_type = parse_optional_return_type(ctx)?;
    ctx.expect(&TokenKind::Assign)?;
    let llvm_name = ctx.expect_string_literal()?;
    ctx.expect(&TokenKind::Semicolon)?;
    let end = ctx.prev_span();
    let span = start_tok.to(end);

    Ok(Stmt::FuncDef(crate::ast::FuncDef {
        name,
        receiver: None,
        params,
        return_type,
        body: FuncBody::Intrinsic(llvm_name),
        span,
    }))
}

pub(super) fn parse_parameter_list(ctx: &mut ParserCtx) -> ParseResult<Vec<(ParamKind, Types)>> {
    ctx.expect(&TokenKind::LeftParen)?;

    let mut params = Vec::new();

    if ctx.peek().is_some_and(|t| t.kind().eq(&TokenKind::Self_)) {
        ctx.expect_keyword(&TokenKind::Self_)?;
        let name = ParamKind::Self_(Selff::new(ctx.current_span()));
        ctx.expect(&TokenKind::Colon)?;
        let param_type = ctx.expect_type()?;
        params.push((name, param_type));
        ctx.consume_if(&TokenKind::Comma);
    }

    let mut rest = ctx.parse_comma_separated(
        |c| {
            if c.peek().is_some_and(|t| t.kind().eq(&TokenKind::Self_)) {
                let span = c.current_span().into_range();
                let source_id = c.source_id.clone();
                return Err(crate::parser::error::ParseError::unexpected_token(
                    c.peek().cloned(),
                    "parameter name",
                    span,
                    source_id,
                ));
            }
            let name = c.expect_identifier()?;
            c.expect(&TokenKind::Colon)?;
            let param_type = c.expect_type()?;
            Ok((ParamKind::Ident(name), param_type))
        },
        &TokenKind::RightParen,
    )?;
    params.append(&mut rest);

    ctx.expect(&TokenKind::RightParen)?;
    Ok(params)
}

pub(super) fn parse_parameter_list_with_variadic(
    ctx: &mut ParserCtx,
) -> ParseResult<(Vec<(ParamKind, Types)>, bool)> {
    ctx.expect(&TokenKind::LeftParen)?;

    let mut params = Vec::new();
    let mut variadic = false;

    let mut rest = ctx.parse_comma_separated_with_variadic(
        |c| {
            let name = c.expect_identifier()?;
            c.expect(&TokenKind::Colon)?;
            let param_type = c.expect_type()?;
            Ok((ParamKind::Ident(name), param_type))
        },
        &mut variadic,
    )?;
    params.append(&mut rest);

    ctx.expect(&TokenKind::RightParen)?;
    Ok((params, variadic))
}

pub(super) fn parse_optional_return_type(ctx: &mut ParserCtx) -> ParseResult<Types> {
    if ctx.consume_if(&TokenKind::Colon) {
        ctx.expect_type()
    } else {
        Ok(Types::Void)
    }
}
