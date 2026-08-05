use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;

pub trait Visit {
    type Output<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>>;
}
