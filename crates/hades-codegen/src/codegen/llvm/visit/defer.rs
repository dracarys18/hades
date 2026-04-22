use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use hades_ast::TypedDefer;

impl Visit for TypedDefer {
    type Output<'ctx> = ();
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let current_fn = context.current_fn_mut_unchecked();
        current_fn.push_defer(self.clone());
        Ok(())
    }
}
