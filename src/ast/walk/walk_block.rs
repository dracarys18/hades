use crate::ast::{Block, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedBlock};

impl WalkAst for Block {
    type Output = TypedBlock;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        ctx.enter_scope();
        let typed_stmts = self.stmts.walk(ctx)?;
        ctx.exit_scope();
        Ok(TypedBlock {
            stmts: typed_stmts,
            span: self.span().clone(),
        })
    }
}
