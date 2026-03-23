use crate::ast::{QualifiedCall, WalkAst};
use crate::error::{SemanticError, Span};
use crate::tokens::FunctionName;
use crate::typed_ast::{CompilerContext, TypedExpr};

use super::func::{qualified_or_bare, walk_typed_args};

impl WalkAst for QualifiedCall {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let resolved = if ctx.structs().fields(&self.qualifier).is_some() {
            qualified_or_bare(&self.func.mangle(&self.qualifier), ctx)
        } else {
            FunctionName::new(
                format!("{}__{}", self.qualifier.inner(), self.func.inner()),
                self.func.span().clone(),
            )
        };
        let sig = ctx.get_function_signature(&resolved)?;
        let return_type = sig.return_type().clone();
        let params = sig.params();
        sig.check_arg_count(self.args.len())
            .then(|| ())
            .ok_or_else(|| {
                SemanticError::argument_count_mismatch(
                    sig.param_count(),
                    self.args.len(),
                    resolved.to_ident(),
                    span.clone(),
                )
            })?;
        walk_typed_args(&params, &self.args, ctx, span).map(|typed_args| TypedExpr::Call {
            func: resolved,
            args: typed_args,
            receiver: None,
            typ: return_type,
        })
    }
}
