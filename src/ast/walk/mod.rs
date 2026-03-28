pub mod call;
mod walk_block;
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

use crate::ast::{Expr, NullExpr, Types};
use crate::error::{SemanticError, Span};
use crate::typed_ast::*;

pub trait WalkAst {
    type Output;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError>;
}

/// Walk an expression that may be `null`, supplying the expected pointer type.
/// If `expr` is `Expr::Null`, delegates to `NullExpr` with the given hint.
/// Otherwise falls through to the normal walk.
pub(super) fn walk_possibly_null(
    expr: &Expr,
    expected: Option<Types>,
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<TypedExpr, SemanticError> {
    if let Expr::Null = expr {
        NullExpr::new(expected).walk(ctx, span)
    } else {
        expr.walk(ctx, span)
    }
}
