use crate::ast::{Import, WalkAst};
use crate::typed_ast::{CompilerContext, TypedImport};

impl WalkAst for Import {
    type Output = TypedImport;
    fn walk(
        &self,
        _ctx: &mut CompilerContext,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        Ok(TypedImport {
            module: self.module.clone(),
            prefix: match self.prefix {
                crate::ast::ImportPrefix::Std => "std".to_string(),
                crate::ast::ImportPrefix::Local => "self".to_string(),
            },
            span: self.span.clone(),
        })
    }
}
