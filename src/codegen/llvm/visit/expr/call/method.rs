use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedExpr;
use inkwell::values::BasicMetadataValueEnum;

use super::build_call;

pub struct MethodCall<'a> {
    pub name: &'a str,
    pub receiver: &'a TypedExpr,
    pub args: &'a [TypedExpr],
}

impl Visit for MethodCall<'_> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let self_ptr = context.get_ptr(self.receiver)?;
        let arg_values = std::iter::once(Ok(self_ptr.into()))
            .chain(
                self.args
                    .iter()
                    .flat_map(|a| a.visit(context).map(|v| v.value().map(|v| v.into()))),
            )
            .collect::<CodegenResult<Vec<BasicMetadataValueEnum>>>()?;
        build_call(self.name, &arg_values, context)
    }
}
