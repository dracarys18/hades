use crate::ast::{StructDef, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::TypedStructDef;

impl WalkAst for StructDef {
    type Output = TypedStructDef;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let name = self.name.clone();
        let fields = self.fields.clone();

        ctx.insert_struct(name.clone(), fields.clone());
        Ok(TypedStructDef {
            name: name.clone(),
            fields: fields.clone(),
            span: self.span.clone(),
        })
    }
}
