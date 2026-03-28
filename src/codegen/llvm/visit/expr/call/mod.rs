pub mod func;
pub mod method;

pub use func::FunctionCall;
pub use method::MethodCall;

use crate::codegen::BuiltinRegistar;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use inkwell::values::BasicMetadataValueEnum;

pub fn build_call<'ctx>(
    name: &str,
    arg_values: &[BasicMetadataValueEnum<'ctx>],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let name_fn = crate::tokens::FunctionName::new(name.to_string(), Default::default());
    let return_type = context
        .symbols()
        .get_function_signature(&name_fn)
        .map_err(|_| CodegenError::FunctionNotFound {
            name: name.to_string(),
        })?
        .return_type()
        .clone();

    if BuiltinRegistar::is_builtin_function(name) {
        return BuiltinRegistar::handle(name, context, arg_values)
            .map(|v| CodegenValue::new(v.try_into().unwrap(), return_type))
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to generate builtin call to {name}"),
            });
    }

    let function = context.get_function(name)?;
    let call_site = context
        .builder()
        .build_call(function, arg_values, "call")
        .map_err(|_| CodegenError::LLVMBuild {
            message: format!("Failed to generate function call to {name}"),
        })?;

    let value = match call_site.try_as_basic_value().basic() {
        Some(v) => v,
        // void call — produce a sentinel i64 0 that callers must not use
        None => context.context().i64_type().const_zero().into(),
    };

    Ok(CodegenValue::new(value, return_type))
}
