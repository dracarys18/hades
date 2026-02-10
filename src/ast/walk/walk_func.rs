use crate::ast::{FuncDef, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, FunctionSignature, TypedFuncDef};

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let function_signature = FunctionSignature::new(
            self.params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            self.return_type.clone(),
        );
        ctx.enter_function(self.name.clone(), function_signature.clone())?;

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
