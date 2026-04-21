use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedContinue;

impl Visit for TypedContinue {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_ctx = context
            .current_loop()
            .ok_or_else(|| CodegenError::LLVMBuild {
                message: "Continue statement outside of loop".to_string(),
            })?;

        let continue_block = loop_ctx.continue_block;
        context.build_unconditional_branch(continue_block)?;
        Ok(())
    }
}
