use crate::ast::Types;
use crate::ast::{For, WalkAst};
use crate::typed_ast::TypedFor;

const ALLOWED_FOR_TYPES: [Types; 2] = [Types::Int, Types::Float];

impl WalkAst for For {
    type Output = TypedFor;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        ctx.enter_scope();
        let typed_init = self.init.walk(ctx, self.span)?;
        let typed_cond = self.cond.walk(ctx, self.span)?;
        let typed_update = self.update.walk(ctx, self.span)?;

        if !ALLOWED_FOR_TYPES.contains(&typed_init.typ) {
            return Err(crate::error::SemanticError::invalid_type(
                typed_init.name.clone(),
                self.span,
            ));
        }

        let typed_body = self.body.walk(ctx, self.span)?;
        ctx.exit_scope();
        Ok(TypedFor {
            init: typed_init,
            cond: typed_cond,
            update: typed_update,
            body: typed_body,
            span: self.span,
        })
    }
}
