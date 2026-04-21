use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedBlock;

impl Visit for TypedBlock {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        for stmt in &self.stmts.0 {
            stmt.visit(context)?;
            if context.is_block_terminated() {
                break;
            }
        }
        Ok(())
    }
}
