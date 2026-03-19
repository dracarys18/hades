use crate::ast::{FuncDef, Types, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, FunctionSignature, TypedFuncDef};

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        // Build the function signature. For methods, params does NOT include `self` —
        // `self` is handled as an implicit first argument at codegen time.
        let function_signature = FunctionSignature::new(
            self.params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            self.parent_struct.clone(),
            self.return_type.clone(),
        );
        ctx.enter_function(self.name.clone(), function_signature.clone())?;

        // If this is a method, insert `self` into scope as the struct type.
        if let Some(ref struct_name) = self.parent_struct {
            let self_ident = crate::tokens::Ident::new("self".to_string(), self.span.clone());
            ctx.insert_variable(self_ident, Types::Struct(struct_name.clone()));
        }

        for (param_name, param_type) in &self.params {
            ctx.insert_variable(param_name.clone(), param_type.clone());
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
