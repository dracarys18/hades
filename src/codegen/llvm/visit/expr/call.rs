use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::BuiltinRegistar;
use crate::typed_ast::TypedExpr;
use inkwell::values::BasicMetadataValueEnum;

pub fn visit_function_call<'ctx>(
    name: &str,
    args: &[TypedExpr],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let function = context.get_function(name)?;
    let arg_values = args
        .iter()
        .map(|a| a.visit(context).map(|v| v.value.into()))
        .collect::<CodegenResult<Vec<BasicMetadataValueEnum>>>()?;
    build_call(name, &arg_values, context)
}

pub fn visit_method_call<'ctx>(
    name: &str,
    receiver: &TypedExpr,
    args: &[TypedExpr],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let self_ptr = context.get_ptr(receiver)?;
    let arg_values = std::iter::once(Ok(self_ptr.into()))
        .chain(
            args.iter()
                .map(|a| a.visit(context).map(|v| v.value.into())),
        )
        .collect::<CodegenResult<Vec<BasicMetadataValueEnum>>>()?;
    build_call(name, &arg_values, context)
}

fn build_call<'ctx>(
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
    context
        .builder()
        .build_call(function, arg_values, "call")
        .map_err(|_| CodegenError::LLVMBuild {
            message: format!("Failed to generate function call to {name}"),
        })
        .map(|r| CodegenValue::new(r.try_as_basic_value().unwrap_basic(), return_type))
}
