use super::Builtin;
use crate::codegen::{
    LLVMContext,
    error::{CodegenError, CodegenResult},
};
use inkwell::AddressSpace;
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum, FunctionValue};

pub struct Printf;

impl Builtin for Printf {
    fn declare<'ctx>(context: &mut LLVMContext<'ctx>) -> FunctionValue<'ctx> {
        let ptr_type = context.context().ptr_type(AddressSpace::default());
        let printf_type = context
            .context()
            .i32_type()
            .fn_type(&[ptr_type.into()], true);

        let printf_fn = context.module().add_function("printf", printf_type, None);

        printf_fn.set_linkage(inkwell::module::Linkage::External);

        printf_fn
    }

    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<AnyValueEnum<'ctx>> {
        if args.is_empty() {
            return Err(CodegenError::LLVMBuild {
                message: "printf requires at least one argument (format string)".to_string(),
            });
        }

        let builder = context.builder();

        let printf_fn = context
            .module()
            .get_function("printf")
            .expect("printf function not found");

        let call_result = builder
            .build_call(printf_fn, args, "printf_call")
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to build printf call".to_string(),
            })?;

        Ok(call_result
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| context.context().i32_type().const_zero().into())
            .into())
    }
}
