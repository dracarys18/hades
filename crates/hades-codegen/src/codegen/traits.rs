use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenError;
use hades_mir::visit::Backend;

impl Backend for LLVMContext<'_> {
    type Error = CodegenError;
}
