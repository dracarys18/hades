use crate::ast::{Let, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedLet};

impl WalkAst for Let {
    type Output = TypedLet;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        let typed_value = self.value.walk(ctx)?;

        let declared_type = self.declared_type.as_ref();
        let inferred_type = typed_value.get_type();

        let span = &self.span;
        let name = &self.name;

        let final_type = match declared_type {
            Some(declared) => {
                if declared != &inferred_type {
                    return Err(SemanticError::TypeMismatch {
                        expected: declared.to_string(),
                        found: inferred_type.to_string(),
                        span: span.clone(),
                    });
                }
                declared.clone()
            }
            None => inferred_type,
        };

        ctx.insert_variable(name.clone(), final_type.clone());
        Ok(TypedLet {
            name: name.clone(),
            typ: final_type,
            value: typed_value,
            span: span.clone(),
        })
    }
}
