use crate::ast::{Let, Types, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedLet};

impl WalkAst for Let {
    type Output = TypedLet;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let typed_value = self.value.walk(ctx, self.span.clone())?;

        let declared_type = self.declared_type.as_ref();
        let inferred_type = typed_value.get_type();

        let span = &self.span;
        let name = &self.name;

        let final_type = match declared_type {
            Some(declared) => {
                if declared != &inferred_type {
                    return Err(SemanticError::type_mismatch(
                        declared.to_string(),
                        inferred_type.to_string(),
                        span.clone(),
                    ));
                }
                declared.clone()
            }
            None => inferred_type,
        };

        if final_type.eq(&Types::Void) {
            return Err(SemanticError::invalid_type(name.clone(), span.clone()));
        }

        ctx.insert_variable(name.clone(), final_type.clone());
        Ok(TypedLet {
            name: name.clone(),
            typ: final_type,
            value: typed_value,
            span: span.clone(),
        })
    }
}
