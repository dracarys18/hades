use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::BuiltinRegistar;
use crate::typed_ast::TypedExpr;

pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: &'a [TypedExpr],
}

impl<'a> FunctionCall<'a> {
    pub fn new(name: &'a str, args: &'a [TypedExpr]) -> Self {
        Self { name, args }
    }
}

impl<'a> Visit for FunctionCall<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let function = context.get_function(self.name)?;

        let mut arg_values = Vec::new();
        for arg in self.args {
            let arg_val = arg.visit(context)?;
            arg_values.push(arg_val.value.into());
        }

        let name_ident = crate::tokens::Ident::new(self.name.to_string(), Default::default());
        let return_type = context
            .symbols()
            .get_function_signature(&name_ident)
            .map_err(|_| CodegenError::FunctionNotFound {
                name: self.name.to_string(),
            })?
            .return_type()
            .clone();

        if BuiltinRegistar::is_builtin_function(self.name) {
            let call_result =
                BuiltinRegistar::handle(self.name, context, &arg_values).map_err(|_| {
                    CodegenError::LLVMBuild {
                        message: format!("Failed to generate function call to {}", self.name),
                    }
                })?;

            return Ok(CodegenValue::new(
                call_result.try_into().unwrap(),
                return_type,
            ));
        }

        let call_result = context
            .builder()
            .build_call(function, &arg_values, "call")
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to generate function call to {}", self.name),
            })?;

        Ok(CodegenValue::new(
            call_result.try_as_basic_value().unwrap_left(),
            return_type,
        ))
    }
}
