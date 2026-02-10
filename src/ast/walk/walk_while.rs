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
        let typed_cond = self.cond.walk(ctx, self.span.clone())?;
        if typed_cond.get_type() != Types::Bool {
            return Err(SemanticError::type_mismatch(
                Types::Bool.to_string(),
                typed_cond.get_type().to_string(),
                self.span.clone(),
            ));
        }

        let typed_body = self.body.walk(ctx, self.span.clone())?;
        Ok(TypedWhile {
            cond: typed_cond,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
