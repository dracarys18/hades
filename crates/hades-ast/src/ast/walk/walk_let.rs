use crate::ast::{Let, Types, WalkAst};
use crate::typed_ast::{CompilerContext, TypedExprAst, TypedLet};
use hades_error::{SemanticError, Span};

use super::walk_possibly_null;

impl WalkAst for Let {
    type Output = TypedLet;

    fn walk(&self, ctx: &mut CompilerContext, _span: Span) -> Result<Self::Output, SemanticError> {
        let span = &self.span;
        let name = &self.name;

        let typed_expr = walk_possibly_null(
            &self.value.expr,
            self.declared_type
                .as_ref()
                .map(|t| t.qualify(ctx.module_name())),
            ctx,
            span.clone(),
        )?;
        let inferred_type = typed_expr.get_type();

        let final_type = match self.declared_type.as_ref() {
            Some(declared) => {
                let qualified = declared.qualify(ctx.module_name());
                if qualified != inferred_type {
                    return Err(SemanticError::type_mismatch(
                        qualified.to_string(),
                        inferred_type.to_string(),
                        span.clone(),
                    ));
                }
                qualified
            }
            None => inferred_type,
        };

        if final_type == Types::Void {
            return Err(SemanticError::invalid_type(
                name.inner().to_string(),
                span.clone(),
            ));
        }

        ctx.insert_variable(name.clone(), final_type.clone());
        Ok(TypedLet {
            name: name.clone(),
            typ: final_type,
            value: TypedExprAst {
                expr: typed_expr,
                span: span.clone(),
            },
            span: span.clone(),
        })
    }
}
