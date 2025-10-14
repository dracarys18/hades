use crate::ast::{Continue, WalkAst};
use crate::typed_ast::{CompilerContext, TypedContinue};

impl WalkAst for Continue {
    type Output = TypedContinue;
    fn walk(
        &self,
        _ctx: &mut CompilerContext,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        Ok(TypedContinue {
            span: self.span.clone(),
        })
    }
}
