use crate::ast::{NullExpr, Types, WalkAst};
use hades_error::{SemanticError, Span};
use crate::typed_ast::{CompilerContext, TypedExpr};

impl WalkAst for NullExpr {
    type Output = TypedExpr;

    fn walk(&self, _ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let expected = self
            .expected
            .as_ref()
            .ok_or_else(|| SemanticError::null_without_type(span.clone()))?;

        match expected {
            Types::Pointer(_) => Ok(TypedExpr::Null(expected.clone())),
            other => Err(SemanticError::null_non_pointer(other.to_string(), span)),
        }
    }
}
