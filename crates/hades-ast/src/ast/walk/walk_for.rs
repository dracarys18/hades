use crate::ast::{For, WalkAst};
use crate::typed_ast::TypedFor;

impl WalkAst for For {
    type Output = TypedFor;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        _span: hades_error::Span,
    ) -> Result<Self::Output, hades_error::SemanticError> {
        ctx.enter_scope();
        let typed_init = self.init.walk(ctx, self.span.clone())?;
        let typed_cond = self.cond.walk(ctx, self.span.clone())?;
        let typed_update = self.update.walk(ctx, self.span.clone())?;

        let typed_body = self.body.walk(ctx, self.span.clone())?;
        ctx.exit_scope();
        Ok(TypedFor {
            init: typed_init,
            cond: typed_cond,
            update: typed_update,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
