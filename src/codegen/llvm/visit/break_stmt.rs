use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedBreak;

impl Visit for TypedBreak {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_ctx = context
            .current_loop()
            .ok_or_else(|| CodegenError::LLVMBuild {
                message: "Break statement outside of loop".to_string(),
            })?;

        let break_block = loop_ctx.break_block;
        context.build_unconditional_branch(break_block)?;
        Ok(())
    }
}
