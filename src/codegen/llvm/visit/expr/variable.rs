use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::Ident;

pub struct VariableAccess<'a> {
    pub name: &'a Ident,
}

impl<'a> VariableAccess<'a> {
    pub fn new(name: &'a Ident) -> Self {
        Self { name }
    }
}

impl<'a> Visit for VariableAccess<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let var = context.get_variable(self.name)?;
        let var_ptr = var.value();
        let var_type = var.typ();

        let symbols = context.symbols();
        let llvm_type = context.type_converter().to_llvm_type(var_type, symbols)?;

        let loaded_val = context.create_load(var_ptr, llvm_type, self.name.inner())?;
        Ok(CodegenValue::new(loaded_val, var_type.clone()))
    }
}
