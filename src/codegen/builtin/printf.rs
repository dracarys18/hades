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
        let print_type = context
            .context()
            .i64_type()
            .fn_type(&[ptr_type.into()], false);
        context.module().add_function("printf", print_type, None)
    }

    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<AnyValueEnum<'ctx>> {
        if args.is_empty() {
            return Err(CodegenError::LLVMBuild {
                message: "print requires at least one argument".to_string(),
            });
        }

        let printf_fn = context
            .module()
            .get_function("printf")
            .expect("printf function not found");

        for (idx, arg) in args.iter().enumerate() {
            let format_str = get_format_string_for_type(arg);
            let format_global = create_format_string_global(context, format_str, idx)?;

            context
                .builder()
                .build_call(
                    printf_fn,
                    &[format_global.as_pointer_value().into(), *arg],
                    "printf_call",
                )
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to call printf".to_string(),
                })?;
        }

        Ok(AnyValueEnum::IntValue(
            context.context().i64_type().const_zero(),
        ))
    }
}

fn get_format_string_for_type(value: &BasicMetadataValueEnum) -> &'static str {
    println!("{:?}", value);
    match value {
        BasicMetadataValueEnum::IntValue(_) => "%lld\n",
        BasicMetadataValueEnum::FloatValue(_) => "%f\n",
        BasicMetadataValueEnum::PointerValue(_) => "%s\n",
        BasicMetadataValueEnum::ArrayValue(_) => "%s\n",
        _ => "%s\n\0",
    }
}

fn create_format_string_global<'ctx>(
    context: &LLVMContext<'ctx>,
    format_str: &str,
    index: usize,
) -> CodegenResult<inkwell::values::GlobalValue<'ctx>> {
    context
        .builder()
        .build_global_string_ptr(format_str, &format!("fmt_str_{}", index))
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create format string global".to_string(),
        })
}
