use crate::ast::ArrayLiteral;
use crate::ast::Types;
use crate::ast::Value;
use crate::ast::WalkAst;
use crate::error::SemanticError;
use crate::error::Span;
use crate::typed_ast::{CompilerContext, TypedArrayLiteral, TypedValue};

impl WalkAst for Value {
    type Output = TypedValue;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        Ok(match self {
            Self::Float(f) => TypedValue::Float(*f),
            Self::Number(n) => TypedValue::Number(*n),
            Self::String(s) => TypedValue::String(s.clone()),
            Self::Boolean(b) => TypedValue::Boolean(*b),
            Self::Array(a) => TypedValue::Array(a.walk(ctx, span)?),
        })
    }
}

impl WalkAst for ArrayLiteral {
    type Output = TypedArrayLiteral;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let typed_expr = self
            .elem
            .iter()
            .map(|e| e.walk(ctx, span))
            .collect::<Result<Vec<_>, _>>()?;

        let expected_type = typed_expr
            .first()
            .expect("Empty arrays are not supported yet")
            .get_type();

        for expr in &typed_expr {
            if !expr.get_type().eq(&expected_type) {
                return Err(SemanticError::TypeMismatch {
                    expected: expected_type.to_string(),
                    found: expr.get_type().to_string(),
                    span,
                });
            }
        }

        Ok(TypedArrayLiteral {
            elements: typed_expr,
            size: self.size,
            elem_typ: Types::Array(expected_type.array_type(self.size)),
        })
    }
}
