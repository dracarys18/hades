use crate::ast::{Return, Types, WalkAst};
use crate::typed_ast::{CompilerContext, TypedReturn};

impl WalkAst for Return {
    type Output = TypedReturn;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, crate::error::SemanticError> {
        let expr = self.expr.as_ref();
        let span = self.span.clone();
        let typed_expr = match expr {
            Some(e) => Some(e.walk(ctx)?),
            None => None,
        };

        let return_type = match &typed_expr {
            Some(e) => e.get_type(),
            None => Types::Void,
        };

        ctx.check_return_type(return_type, span.clone())?;
        Ok(TypedReturn {
            expr: typed_expr,
            span,
        })
    }
}
