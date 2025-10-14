use crate::ast::{For, Types, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::TypedFor;

impl WalkAst for For {
    type Output = TypedFor;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_init = self.init.walk(ctx)?;
        let typed_cond = self.cond.walk(ctx)?;
        let typed_update = self.update.walk(ctx)?;

        if typed_cond.get_type() != Types::Bool {
            return Err(SemanticError::TypeMismatch {
                expected: Types::Bool.to_string(),
                found: typed_cond.get_type().to_string(),
                span: self.span,
            });
        }

        let typed_body = self.body.walk(ctx)?;
        Ok(TypedFor {
            init: typed_init,
            cond: typed_cond,
            update: typed_update,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
