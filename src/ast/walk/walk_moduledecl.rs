use crate::ast::ModuleDecl;
use crate::ast::WalkAst;
use crate::error::SemanticError;
use crate::error::Span;
use crate::typed_ast::{CompilerContext, TypedModuleDecl};

const DISALLOWED_MODULE_NAMES: &[&str] = &["main", "std", "core"];

impl WalkAst for ModuleDecl {
    type Output = TypedModuleDecl;

    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        if DISALLOWED_MODULE_NAMES.contains(&self.name.to_string().as_str()) {
            return Err(SemanticError::InvalidModuleName {
                name: self.name.clone(),
                span: self.span,
            });
        }

        Ok(TypedModuleDecl {
            name: self.name.clone(),
            span: self.span,
        })
    }
}
