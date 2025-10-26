use inkwell::values::BasicValueEnum;

use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::{context::LLVMContext, llvm::visit::expr::variable::VariableAccess};
use crate::tokens::{Ident, Op};
use crate::typed_ast::TypedExpr;

pub struct Assignment<'a> {
    pub name: &'a Ident,
    pub op: &'a Op,
    pub value: &'a TypedExpr,
}

impl<'a> Assignment<'a> {
    pub fn new(name: &'a Ident, op: &'a Op, value: &'a TypedExpr) -> Self {
        Self { name, op, value }
    }
}

impl<'a> Visit for Assignment<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let value_val = self.value.visit(context)?;
        let var_ptr = context.get_variable(self.name)?;

        let current_value = VariableAccess::new(self.name).visit(context)?;
        match self.op {
            Op::PlusEqual => {
                let new_value = generate_add(context, current_value.value, value_val.value)?;
                context
                    .builder()
                    .build_store(var_ptr.value(), new_value)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;

                Ok(CodegenValue {
                    value: new_value,
                    type_info: value_val.type_info,
                })
            }
            Op::MinusEqual => {
                let new_value = generic_sub(context, current_value.value, value_val.value)?;
                context
                    .builder()
                    .build_store(var_ptr.value(), new_value)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;
                Ok(CodegenValue {
                    value: new_value,
                    type_info: value_val.type_info,
                })
            }
            Op::Assign => {
                context
                    .builder()
                    .build_store(var_ptr.value(), value_val.value)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;

                Ok(CodegenValue {
                    value: value_val.value,
                    type_info: value_val.type_info,
                })
            }
            _ => Err(CodegenError::LLVMBuild {
                message: format!("Unsupported assignment operator: {:?}", self.op),
            }),
        }
    }
}

fn generate_add<'a>(
    context: &mut LLVMContext<'a>,
    value: BasicValueEnum<'a>,
    add: BasicValueEnum<'a>,
) -> CodegenResult<BasicValueEnum<'a>> {
    match value {
        BasicValueEnum::IntValue(curr) => {
            let new_value = context
                .builder()
                .build_int_add(curr, add.into_int_value(), "add_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::IntValue(new_value))
        }
        BasicValueEnum::FloatValue(curr) => {
            let new_value = context
                .builder()
                .build_float_add(curr, add.into_float_value(), "add_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::FloatValue(new_value))
        }
        _ => Ok(value),
    }
}

fn generic_sub<'a>(
    context: &mut LLVMContext<'a>,
    value: BasicValueEnum<'a>,
    add: BasicValueEnum<'a>,
) -> CodegenResult<BasicValueEnum<'a>> {
    match value {
        BasicValueEnum::IntValue(curr) => {
            let new_value = context
                .builder()
                .build_int_sub(curr, add.into_int_value(), "add_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::IntValue(new_value))
        }
        BasicValueEnum::FloatValue(curr) => {
            let new_value = context
                .builder()
                .build_float_sub(curr, add.into_float_value(), "add_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::FloatValue(new_value))
        }
        _ => Ok(value),
    }
}
