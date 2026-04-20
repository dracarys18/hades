use crate::codegen::error::CodegenError;
use crate::codegen::{LLVMContext, error::CodegenResult, traits::Visit};
use crate::typed_ast::TypedDefer;

impl Visit for TypedDefer {
    type Output<'ctx> = ();
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let current_fn = context.current_fn_mut_unchecked();
        current_fn.push_defer(self.clone());
        Ok(())
    }
}
