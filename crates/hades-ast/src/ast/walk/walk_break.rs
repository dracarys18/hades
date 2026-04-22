use crate::ast::{Break, WalkAst};
use hades_error::{SemanticError, Span};
use crate::typed_ast::{CompilerContext, TypedBreak};

impl WalkAst for Break {
    type Output = TypedBreak;
    fn walk(&self, _ctx: &mut CompilerContext, _span: Span) -> Result<Self::Output, SemanticError> {
        Ok(TypedBreak {
            span: self.span.clone(),
        })
    }
}
