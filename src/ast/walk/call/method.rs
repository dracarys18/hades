use crate::ast::{MethodCall, WalkAst};
use crate::error::{SemanticError, Span};
use crate::tokens::Ident;
use crate::typed_ast::{CompilerContext, TypedExpr};

use super::func::walk_typed_args;

impl WalkAst for MethodCall {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let typed_receiver = self.receiver.walk(ctx, span.clone())?;
        let receiver_type = typed_receiver.get_type();
        let struct_name = receiver_type.unwrap_struct_name();
        let bare_struct_ident = Ident::new(struct_name.link_name().to_string(), span.clone());
        let mangled = self.func.mangle(&bare_struct_ident);

        let resolved = if let Some(struct_module) = struct_name.module() {
            mangled.full_name(struct_module)
        } else {
            ctx.module_name()
                .map(|m| mangled.full_name(m))
                .filter(|n| ctx.get_function_signature(n).is_ok())
                .unwrap_or_else(|| mangled.clone())
        };
        let sig = ctx.get_function_signature(&resolved)?;
        let return_type = sig.return_type().clone();
        let params = sig.params();
        sig.check_arg_count(self.args.len())
            .then_some(())
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
            receiver: Some(Box::new(typed_receiver)),
            typ: return_type,
        })
    }
}
