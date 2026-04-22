use crate::ast::{Continue, WalkAst};
use crate::typed_ast::{CompilerContext, TypedContinue};

impl WalkAst for Continue {
    type Output = TypedContinue;
    fn walk(
        &self,
        _ctx: &mut CompilerContext,
        _span: hades_error::Span,
    ) -> Result<Self::Output, hades_error::SemanticError> {
        Ok(TypedContinue {
            span: self.span.clone(),
        })
    }
}
