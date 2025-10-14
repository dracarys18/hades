use crate::ast::{FuncDef, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedFuncDef};

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        ctx.enter_function(
            self.name.clone(),
            self.params.clone(),
            self.return_type.clone(),
        )?;

        for (param_name, param_type) in &self.params {
            ctx.insert_variable(param_name.clone(), param_type.clone());
        }

        let typed_body = self.body.walk(ctx)?;
        ctx.exit_function();
        Ok(TypedFuncDef {
            name: self.name.clone(),
            params: self.params.clone(),
            return_type: self.return_type.clone(),
            body: typed_body,
            span: self.span,
        })
    }
}
