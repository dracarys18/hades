use crate::parser::Parse;
use crate::parser::ParserCtx;
use crate::parser::error::ParseResult;
use hades_ast::{Import, ImportPrefix, ModuleDecl, Stmt};
use hades_common::token_matches;
use hades_tokens::TokenKind;

impl Parse for ModuleDecl {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Module)?;
        let name = ctx.expect_identifier()?;
        let end = ctx.prev_span();

        Ok(Stmt::ModuleDecl(ModuleDecl {
            name,
            span: start_tok.to(end),
        }))
    }
}

impl Parse for Import {
    type Output = Stmt;

    fn parse(ctx: &mut ParserCtx) -> ParseResult<Stmt> {
        let start_tok = ctx.current_span();
        ctx.expect(&TokenKind::Import)?;

        let prefix = match ctx.peek() {
            Some(tok) if token_matches!(tok, TokenKind::Std) => {
                ctx.next();
                ImportPrefix::Std
            }
            Some(tok) if token_matches!(tok, TokenKind::Self_) => {
                ctx.next();
                ImportPrefix::Local
            }
            _ => {
                return Err(crate::parser::error::ParseError::unexpected_token(
                    ctx.peek().cloned(),
                    "std or self",
                    ctx.current_span().into_range(),
                    ctx.source_id.clone(),
                ));
            }
        };

        ctx.expect(&TokenKind::DoubleColon)?;
        let module_name = ctx.expect_identifier()?;
        let end = ctx.prev_span();

        Ok(Stmt::Import(Import {
            prefix,
            module: module_name.inner().to_string(),
            span: start_tok.to(end),
        }))
    }
}
