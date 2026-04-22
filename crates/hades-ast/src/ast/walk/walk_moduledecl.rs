use crate::ast::ModuleDecl;
use crate::ast::WalkAst;
use hades_common::consts::ENTRY_POINT;
use hades_error::SemanticError;
use hades_error::Span;
use crate::typed_ast::{CompilerContext, TypedModuleDecl};

const DISALLOWED_MODULE_NAMES: &[&str] = &[ENTRY_POINT, "std", "core"];

impl WalkAst for ModuleDecl {
    type Output = TypedModuleDecl;

    fn walk(&self, _ctx: &mut CompilerContext, _span: Span) -> Result<Self::Output, SemanticError> {
        if DISALLOWED_MODULE_NAMES.contains(&self.name.to_string().as_str()) {
            return Err(SemanticError::invalid_module_name(
                self.name.inner().to_string(),
                self.span.clone(),
            ));
        }

        Ok(TypedModuleDecl {
            name: self.name.clone(),
            span: self.span.clone(),
        })
    }
}
