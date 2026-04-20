use crate::{
    ast::{DeferStmt, Stmt, WalkAst},
    typed_ast::TypedDefer,
};

impl WalkAst for DeferStmt {
    type Output = TypedDefer;

    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        if ctx.current_function().is_none() {
            return Err(crate::error::SemanticError::defer_outside_function(span));
        }

        self.stmt.walk(ctx, span).map(|stmt| TypedDefer {
            stmt: stmt,
            span: self.span.clone(),
        })
    }
}
