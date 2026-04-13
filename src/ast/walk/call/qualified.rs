use crate::ast::{QualifiedCall, WalkAst};
use crate::error::{SemanticError, Span};
use crate::tokens::Name;
use crate::typed_ast::{CompilerContext, TypedExpr};

use super::func::walk_typed_args;

impl WalkAst for QualifiedCall {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let resolved = match self.path.as_slice() {
            [qualifier] => {
                let struct_key = Name::new(qualifier.to_string(), qualifier.span().clone())
                    .full_name_optional(ctx.module_name());
                if ctx.structs().fields(&struct_key).is_some() {
                    let mangled = self.func.mangle(qualifier);
                    ctx.module_name()
                        .map(|m| mangled.full_name(m))
                        .filter(|n| ctx.get_function_signature(n).is_ok())
                        .unwrap_or_else(|| mangled.clone())
                } else {
                    self.func.full_name(qualifier.inner())
                }
            }
            [module, struct_name] => {
                let mangled = self.func.mangle(struct_name);
                mangled.full_name(module.inner())
            }
            _ => {
                return Err(SemanticError::undefined_function(
                    self.func.to_ident(),
                    span,
                ))
            }
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
