use crate::ast::{Import, WalkAst};
use crate::typed_ast::{CompilerContext, TypedImport};

impl WalkAst for Import {
    type Output = TypedImport;
    fn walk(
        &self,
        _ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        Ok(TypedImport {
            module: self.module.clone(),
            prefix: self.prefix.clone(),
            span: self.span.clone(),
        })
    }
}
