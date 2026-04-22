use hades_ast::{Block, Stmt};
use crate::parser::{ParserCtx, error::ParseResult, stmt};
use hades_tokens::TokenKind::Defer;
use hades_ast::DeferStmt;
use crate::parser::Parse;

impl Parse for DeferStmt {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&Defer)?;
        let block = stmt::parse_block(ctx)?;
        let end = ctx.prev_span();
        Ok(Stmt::Defer(DeferStmt {
            stmt: Block::new(block.into(), end.clone()),
            span: start_tok.to(end),
        }))
    }
}
