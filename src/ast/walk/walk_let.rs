use crate::ast::{Expr, Let, NullExpr, Types, WalkAst};
use crate::error::{SemanticError, Span};
use crate::typed_ast::{CompilerContext, TypedExpr, TypedExprAst, TypedLet};

fn walk_null_rhs(
    declared_type: Option<&Types>,
    span: &Span,
    ctx: &mut CompilerContext,
) -> Result<TypedExpr, SemanticError> {
    NullExpr::new(declared_type.cloned()).walk(ctx, span.clone())
}

impl WalkAst for Let {
    type Output = TypedLet;

    fn walk(&self, ctx: &mut CompilerContext, _span: Span) -> Result<Self::Output, SemanticError> {
        let span = &self.span;
        let name = &self.name;

        if let Expr::Null = &self.value.expr {
            let typed_null = walk_null_rhs(self.declared_type.as_ref(), span, ctx)?;
            let typ = typed_null.get_type();
            ctx.insert_variable(name.clone(), typ.clone());
            return Ok(TypedLet {
                name: name.clone(),
                typ,
                value: TypedExprAst {
                    expr: typed_null,
                    span: span.clone(),
                },
                span: span.clone(),
            });
        }

        let typed_value = self.value.walk(ctx, span.clone())?;
        let inferred_type = typed_value.get_type();

        let final_type = match self.declared_type.as_ref() {
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

        if final_type == Types::Void {
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
