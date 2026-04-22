use crate::ast::{Block, WalkAst};
use hades_error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedBlock};

impl WalkAst for Block {
    type Output = TypedBlock;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: hades_error::Span,
    ) -> Result<Self::Output, SemanticError> {
        ctx.enter_scope();
        let typed_stmts = self.stmts.walk(ctx, span)?;
        ctx.exit_scope();
        Ok(TypedBlock {
            stmts: typed_stmts,
            span: self.span().clone(),
        })
    }
}
