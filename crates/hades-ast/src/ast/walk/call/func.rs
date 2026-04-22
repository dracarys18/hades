use crate::ast::{Expr, FunctionCall, WalkAst};
use hades_error::{SemanticError, Span};
use crate::typed_ast::{CompilerContext, TypedExpr};

impl WalkAst for FunctionCall {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let resolved = ctx
            .module_name()
            .map(|m| self.func.full_name(m))
            .filter(|n| ctx.get_function_signature(n).is_ok())
            .unwrap_or_else(|| self.func.clone());
        let sig = ctx.get_function_signature(&resolved)?;
        let return_type = sig.return_type().clone();
        let params = sig.params();
        sig.check_arg_count(self.args.len())
            .then_some(())
            .ok_or_else(|| {
                SemanticError::argument_count_mismatch(
                    sig.param_count(),
                    self.args.len(),
                    resolved.inner().to_string(),
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

pub fn walk_typed_args(
    params: &crate::typed_ast::Params,
    args: &[Expr],
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<Vec<TypedExpr>, SemanticError> {
    args.iter()
        .enumerate()
        .map(|(i, arg)| {
            arg.walk(ctx, span.clone()).and_then(|typed| {
                params
                    .type_match(i, &typed.get_type())
                    .then(|| typed.clone())
                    .ok_or_else(|| {
                        let expected = params.type_at(i).map(|t| t.to_string()).unwrap_or_default();
                        SemanticError::type_mismatch(
                            expected,
                            typed.get_type().to_string(),
                            span.clone(),
                        )
                    })
            })
        })
        .collect()
}
