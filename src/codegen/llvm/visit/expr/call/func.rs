use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::BuiltinRegistar;
use crate::typed_ast::TypedExpr;
use inkwell::values::BasicMetadataValueEnum;

use super::build_call;

pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: &'a [TypedExpr],
}

impl Visit for FunctionCall<'_> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        if BuiltinRegistar::is_compile_time_function(self.name) {
            return BuiltinRegistar::handle_compile_time(self.name, self.args, context);
        }

        let arg_values = self
            .args
            .iter()
            .flat_map(|a| a.visit(context).map(|v| v.value().map(|v| v.into())))
            .collect::<CodegenResult<Vec<BasicMetadataValueEnum>>>()?;
        build_call(self.name, &arg_values, context)
    }
}
