use crate::ast::ArrayLiteral;
use crate::ast::Types;
use crate::ast::Value;
use crate::ast::WalkAst;
use hades_error::SemanticError;
use hades_error::Span;
use crate::typed_ast::{CompilerContext, TypedArrayLiteral, TypedValue};

use super::walk_possibly_null;

impl WalkAst for Value {
    type Output = TypedValue;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        Ok(match self {
            Self::Float(f) => TypedValue::Float(*f),
            Self::Number(n) => TypedValue::Number(*n),
            Self::String(s) => TypedValue::String(s.clone()),
            Self::Boolean(b) => TypedValue::Boolean(*b),
            Self::Array(a) => TypedValue::Array(a.walk(ctx, span)?),
            Self::Char(c) => TypedValue::Char(*c),
        })
    }
}

/// Walk an array literal, optionally providing an expected element type hint.
/// When `elem_hint` is `Some(T)`, each `null` element is resolved to type `T`
/// instead of producing a "null requires explicit type" error.
pub(super) fn walk_array_with_hint(
    arr: &ArrayLiteral,
    elem_hint: Option<Types>,
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<TypedArrayLiteral, SemanticError> {
    let typed_expr = arr
        .elem
        .iter()
        .map(|e| walk_possibly_null(e, elem_hint.clone(), ctx, span.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    let expected_type = typed_expr
        .first()
        .expect("Empty arrays are not supported yet")
        .get_type();

    for expr in &typed_expr {
        if !expr.get_type().eq(&expected_type) {
            return Err(SemanticError::type_mismatch(
                expected_type.to_string(),
                expr.get_type().to_string(),
                span,
            ));
        }
    }

    let typed_fill = arr
        .fill
        .as_deref()
        .map(|f| walk_possibly_null(f, elem_hint.clone(), ctx, span.clone()))
        .transpose()?
        .map(Box::new);

    Ok(TypedArrayLiteral {
        elements: typed_expr,
        size: arr.size,
        elem_typ: Types::Array(expected_type.array_type(arr.size)),
        fill: typed_fill,
    })
}

impl WalkAst for ArrayLiteral {
    type Output = TypedArrayLiteral;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        walk_array_with_hint(self, None, ctx, span)
    }
}
