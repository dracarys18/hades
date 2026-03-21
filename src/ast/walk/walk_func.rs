use crate::ast::{FuncDef, Types, WalkAst};
use crate::error::SemanticError;
use crate::tokens::Ident;
use crate::typed_ast::{CompilerContext, FunctionSignature, TypedFuncDef};
use indexmap::IndexMap;

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        // Separate `self: Self` from regular params.
        // The walker resolves `self`'s placeholder Void type to Struct(parent_struct).
        let (self_param, regular_params): (Vec<_>, Vec<_>) = self
            .params
            .iter()
            .partition(|(name, _)| name.inner() == "self");

        let has_self = !self_param.is_empty();

        // Build the function signature from regular (non-self) params only.
        let params_map: IndexMap<Ident, Types> = regular_params
            .iter()
            .map(|(k, v)| ((*k).clone(), (*v).clone()))
            .collect();

        let function_signature = FunctionSignature::new(
            params_map,
            self.parent_struct.clone(),
            self.return_type.clone(),
        );
        ctx.enter_function(self.name.clone(), function_signature.clone())?;

        // Insert `self` into scope with the resolved struct type.
        if has_self {
            if let Some(ref struct_name) = self.parent_struct {
                let self_ident = Ident::new("self".to_string(), self.span.clone());
                ctx.insert_variable(self_ident, Types::Struct(struct_name.clone()));
            }
        }

        for (param_name, param_type) in &regular_params {
            ctx.insert_variable((*param_name).clone(), (*param_type).clone());
        }

        let typed_body = self.body.walk(ctx, self.span.clone())?;
        ctx.exit_function();

        Ok(TypedFuncDef {
            name: self.name.clone(),
            signature: function_signature,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
