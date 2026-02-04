use inkwell::values::BasicValueEnum;

use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::VisitOptions;
use crate::codegen::{
    context::LLVMContext, llvm::visit::expr::variable::VariableAccess, symbols::LLVMVariable,
};
use crate::tokens::Op;
use crate::typed_ast::{TypedAssignTarget, TypedExpr};

pub struct Assignment<'a> {
    pub target: &'a TypedAssignTarget,
    pub op: &'a Op,
    pub value: &'a TypedExpr,
}

impl<'a> Assignment<'a> {
    pub fn new(target: &'a TypedAssignTarget, op: &'a Op, value: &'a TypedExpr) -> Self {
        Self { target, op, value }
    }

    fn get_target_ptr<'b>(&self, ctx: &mut LLVMContext<'b>) -> CodegenResult<LLVMVariable<'b>> {
        match self.target {
            TypedAssignTarget::Ident(ident) => ctx.get_variable(ident),
            TypedAssignTarget::FieldAccess(field) => {
                let symbols = ctx.symbols();

                let struct_ptr = match field.expr.as_ref() {
                    crate::typed_ast::TypedExpr::Ident { ident, .. } => {
                        ctx.get_variable(ident)?.value()
                    }
                    _ => field.expr.visit(ctx)?.value.into_pointer_value(),
                };

                let struct_type = ctx
                    .type_converter()
                    .to_llvm_type(&field.struct_type, symbols)?;

                let struct_name = field.struct_type.unwrap_struct_name();
                let strct = symbols.structs();
                let field_index = strct.field_index(struct_name, &field.field);

                let zero = ctx.context().i32_type().const_zero();
                let field_index_val = ctx
                    .context()
                    .i32_type()
                    .const_int(field_index as u64, false);

                let field_ptr = unsafe {
                    ctx.builder().build_in_bounds_gep(
                        struct_type,
                        struct_ptr,
                        &[zero, field_index_val],
                        "field_assign_ptr",
                    )
                }
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to create struct field pointer for assignment".to_string(),
                })?;

                Ok(LLVMVariable::new(field_ptr, field.field_type.clone()))
            }
        }
    }
}

impl<'a> Visit for Assignment<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let value_val = self.value.visit(context)?;
        let var_ptr = self.get_target_ptr(context)?;

        let current_value = self.target.visit(context)?;
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

impl Visit for TypedAssignTarget {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Self::Ident(ident) => {
                let current_value =
                    VariableAccess::new(ident, VisitOptions::new()).visit(context)?;
                Ok(current_value)
            }
            Self::FieldAccess(field) => field.visit(context),
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
                .build_int_sub(curr, add.into_int_value(), "sub_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::IntValue(new_value))
        }
        BasicValueEnum::FloatValue(curr) => {
            let new_value = context
                .builder()
                .build_float_sub(curr, add.into_float_value(), "sub_assign")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to build add for assignment: {e:?}"),
                })?;
            Ok(BasicValueEnum::FloatValue(new_value))
        }
        _ => Ok(value),
    }
}
