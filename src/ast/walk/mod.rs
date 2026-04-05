pub mod call;
mod walk_as;
mod walk_block;
mod walk_break;
mod walk_continue;
mod walk_expr;
mod walk_for;
mod walk_func;
mod walk_if;
mod walk_import;
mod walk_let;
mod walk_moduledecl;
mod walk_null;
mod walk_program;
mod walk_return;
mod walk_stmt;
pub mod walk_structdef;
mod walk_value;
mod walk_while;

use crate::ast::{Expr, NullExpr, Types, Value};
use crate::error::{SemanticError, Span};
use crate::typed_ast::*;

pub trait WalkAst {
    type Output;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError>;
}

pub(super) fn walk_possibly_null(
    expr: &Expr,
    expected: Option<Types>,
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<TypedExpr, SemanticError> {
    match expr {
        Expr::Null => NullExpr::new(expected).walk(ctx, span),
        Expr::Value(Value::Array(arr)) => {
            let elem_hint = match &expected {
                Some(t @ Types::Array(_)) => Some(t.get_array_elem_type()),
                _ => None,
            };
            walk_value::walk_array_with_hint(arr, elem_hint, ctx, span)
                .map(|a| TypedExpr::Value(TypedValue::Array(a)))
        }
        _ => expr.walk(ctx, span),
    }
}
