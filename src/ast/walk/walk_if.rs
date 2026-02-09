use crate::ast::If;
use crate::ast::{Types, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedIf};

impl WalkAst for If {
    type Output = TypedIf;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_cond = self.cond.walk(ctx, self.span)?;
        if typed_cond.get_type() != Types::Bool {
            return Err(SemanticError::type_mismatch(
                Types::Bool.to_string(),
                typed_cond.get_type().to_string(),
                self.span,
            ));
        }

        let typed_then = self.then_branch.walk(ctx, self.span)?;
        let typed_else = match self.else_branch {
            Some(ref else_stmts) => Some(else_stmts.walk(ctx, self.span)?),
            None => None,
        };

        Ok(TypedIf {
            cond: typed_cond,
            then_branch: typed_then,
            else_branch: typed_else,
            span: self.span.clone(),
        })
    }
}
