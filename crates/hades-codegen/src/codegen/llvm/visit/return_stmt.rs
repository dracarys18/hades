use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use hades_ast::{TypedBlock, TypedReturn};

impl Visit for TypedReturn {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let defer_stmts: Vec<TypedBlock> = context
            .current_function_unchecked()
            .defer_iter()
            .map(|d| d.stmt.clone())
            .collect();
        for block in defer_stmts {
            block.visit(context)?;
        }

        match &self.expr {
            Some(expr) => {
                let return_val = expr.expr().visit(context)?;
                context.build_return(Some(return_val.value()?))?;
            }
            None => {
                context.build_return(None)?;
            }
        }
        Ok(())
    }
}
