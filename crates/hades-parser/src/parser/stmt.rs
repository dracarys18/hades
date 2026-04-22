use hades_ast::*;
use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::parser::error::ParseResult;
use crate::parser::expr::parse_assignment;
use crate::parser::func::{FuncDef, parse_extern_fn, parse_intrinsic_fn};
use crate::parser::struct_::StructDef;
use hades_common::token_matches;
use hades_tokens::TokenKind;

impl Parse for Stmt {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        match ctx.peek() {
            Some(tok) if token_matches!(tok, TokenKind::Struct) => StructDef::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Fn) => FuncDef::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Extern) => parse_extern_fn(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Intrinsic) => parse_intrinsic_fn(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Let) => Let::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::If) => If::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::While) => While::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::For) => For::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Return) => Return::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Continue) => Continue::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Break) => Break::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Module) => ModuleDecl::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Import) => Import::parse(ctx),
            Some(tok) if token_matches!(tok, TokenKind::Defer) => DeferStmt::parse(ctx),
            _ => parse_expr_stmt(ctx),
        }
    }
}

impl Parse for Let {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Let)?;
        let name = ctx.expect_identifier()?;

        let var_type = if ctx
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::Colon))
        {
            ctx.expect(&TokenKind::Colon)?;
            Some(ctx.expect_type()?)
        } else {
            None
        };

        ctx.expect(&TokenKind::Assign)?;
        let value = parse_assignment(ctx)?;
        ctx.expect(&TokenKind::Semicolon)?;
        let end = ctx.prev_span();
        let span = start_tok.to(end);

        Ok(Stmt::Let(Let {
            name,
            declared_type: var_type,
            value: ExprAst {
                expr: value,
                span: span.clone(),
            },
            span,
        }))
    }
}

impl Parse for If {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::If)?;
        let cond = parse_if_expr(ctx)?;
        let then_branch = parse_stmt_or_block(ctx)?;
        let else_branch = if ctx.consume_if(&TokenKind::Else) {
            Some(parse_stmt_or_block(ctx)?)
        } else {
            None
        };
        let end = ctx.prev_span();
        let span = start_tok.to(end);

        Ok(Stmt::If(If {
            cond: ExprAst {
                expr: cond,
                span: span.clone(),
            },
            then_branch: Block::new(then_branch.into(), span.clone()),
            else_branch: else_branch.map(|p| Block::new(p.into(), span.clone())),
            span,
        }))
    }
}

impl Parse for While {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::While)?;
        let cond = parse_while_expr(ctx)?;
        let body = parse_stmt_or_block(ctx)?;
        let end = ctx.prev_span();
        let span = start_tok.to(end);

        Ok(Stmt::While(While {
            cond,
            body: Block::new(body.into(), span.clone()),
            span,
        }))
    }
}

impl Parse for For {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::For)?;
        let init = Let::parse(ctx)?.unwrap_let();
        let cond = parse_while_expr(ctx)?.unwrap_binary();
        ctx.expect(&TokenKind::Semicolon)?;
        let update = parse_assignment(ctx)?.unwrap_assign();
        let body = parse_stmt_or_block(ctx)?;
        let end = ctx.prev_span();
        let span = start_tok.to(end);

        Ok(Stmt::For(For {
            init,
            cond: cond.clone(),
            update: update.clone(),
            body: Block::new(body.into(), span.clone()),
            span,
        }))
    }
}

impl Parse for Return {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Return)?;
        let expr = if !ctx
            .peek()
            .is_some_and(|tok| token_matches!(tok, TokenKind::Semicolon))
        {
            Some(parse_assignment(ctx)?)
        } else {
            None
        };
        ctx.expect(&TokenKind::Semicolon)?;
        let end = ctx.prev_span();
        let span = start_tok.to(end);
        let expr = expr.map(|e| ExprAst {
            expr: e,
            span: span.clone(),
        });

        Ok(Stmt::Return(Return { expr, span }))
    }
}

impl Parse for Continue {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Continue)?;
        ctx.expect(&TokenKind::Semicolon)?;
        let end = ctx.prev_span();
        Ok(Stmt::Continue(Continue {
            span: start_tok.to(end),
        }))
    }
}

impl Parse for Break {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Break)?;
        ctx.expect(&TokenKind::Semicolon)?;
        let end = ctx.prev_span();
        Ok(Stmt::Break(Break {
            span: start_tok.to(end),
        }))
    }
}

pub(super) fn parse_block(ctx: &mut ParserCtx) -> ParseResult<Vec<Stmt>> {
    ctx.expect(&TokenKind::LeftBrace)?;
    let mut stmts = Vec::new();

    while !ctx
        .peek()
        .is_some_and(|tok| token_matches!(tok, TokenKind::RightBrace))
    {
        stmts.push(Stmt::parse(ctx)?);
    }

    ctx.expect(&TokenKind::RightBrace)?;
    Ok(stmts)
}

pub(super) fn parse_stmt_or_block(ctx: &mut ParserCtx) -> ParseResult<Vec<Stmt>> {
    if ctx
        .peek()
        .is_some_and(|tok| token_matches!(tok, TokenKind::LeftBrace))
    {
        parse_block(ctx)
    } else {
        Ok(vec![Stmt::parse(ctx)?])
    }
}

fn parse_expr_stmt(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
    let start_tok = ctx.current_span();
    let expr = parse_assignment(ctx)?;
    ctx.expect(&TokenKind::Semicolon)?;
    let end = ctx.prev_span();
    Ok(Stmt::Expr(ExprAst {
        expr,
        span: start_tok.to(end),
    }))
}

pub(super) fn parse_if_expr(ctx: &mut ParserCtx) -> ParseResult<Expr> {
    crate::parser::expr::parse_binary_with_flags(ctx, 0, false)
}

pub(super) fn parse_while_expr(ctx: &mut ParserCtx) -> ParseResult<Expr> {
    crate::parser::expr::parse_binary_with_flags(ctx, 0, false)
}
