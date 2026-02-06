use crate::ast::Types;
use crate::ast::{WalkAst, While};
use crate::error::SemanticError;
use crate::typed_ast::TypedWhile;

impl WalkAst for While {
    type Output = TypedWhile;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_cond = self.cond.walk(ctx, self.span)?;
        if typed_cond.get_type() != Types::Bool {
            return Err(SemanticError::TypeMismatch {
                expected: Types::Bool.to_string(),
                found: typed_cond.get_type().to_string(),
                span: self.span,
            });
        }

        let typed_body = self.body.walk(ctx, self.span)?;
        Ok(TypedWhile {
            cond: typed_cond,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
