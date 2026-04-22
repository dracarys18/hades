use crate::{
    ast::{DeferStmt, WalkAst},
    typed_ast::TypedDefer,
};

impl WalkAst for DeferStmt {
    type Output = TypedDefer;

    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: hades_error::Span,
    ) -> Result<Self::Output, hades_error::SemanticError> {
        if ctx.current_function().is_none() {
            return Err(hades_error::SemanticError::defer_outside_function(span));
        }

        self.stmt.walk(ctx, span).map(|stmt| TypedDefer {
            stmt,
            span: self.span.clone(),
        })
    }
}
