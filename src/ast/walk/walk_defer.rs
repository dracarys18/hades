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
        if let Stmt::Return(_) = self.stmt.as_ref() {
            return Err(crate::error::SemanticError::return_not_allowed_in_defer(
                span,
            ));
        }

        if ctx.current_function().is_none() {
            return Err(crate::error::SemanticError::defer_outside_function(span));
        }

        self.stmt.walk(ctx, span).map(|stmt| TypedDefer {
            stmt: Box::new(stmt),
            span: self.span.clone(),
        })
    }
}
