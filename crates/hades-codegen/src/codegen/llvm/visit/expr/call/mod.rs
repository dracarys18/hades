pub mod func;
pub mod method;

pub use func::FunctionCall;
pub use method::MethodCall;

use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use inkwell::values::BasicMetadataValueEnum;

pub fn build_call<'ctx>(
    name: &str,
    arg_values: &[BasicMetadataValueEnum<'ctx>],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let name_fn = hades_tokens::Name::new(name.to_string(), Default::default());
    let sig = context
        .symbols()
        .get_function_signature(&name_fn)
        .map_err(|_| CodegenError::FunctionNotFound {
            name: name.to_string(),
        })?
        .clone();

    let function = context.get_function(name, &sig)?;
    let call_site = context
        .builder()
        .build_call(function, arg_values, "call")
        .map_err(|_| CodegenError::LLVMBuild {
            message: format!("Failed to generate function call to {name}"),
        })?;

    Ok(match call_site.try_as_basic_value().basic() {
        Some(v) => CodegenValue::new(v, sig.return_type().clone()),
        None => CodegenValue::Void,
    })
}
