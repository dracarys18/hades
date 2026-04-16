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

                let raw_ptr = match field.expr.as_ref() {
                    crate::typed_ast::TypedExpr::Ident { ident, .. } => {
                        ctx.get_variable(ident)?.value()
                    }
                    other => ctx.get_ptr(other)?,
                };

                let struct_ptr = ctx.deref_if_pointer(raw_ptr, &field.expr.get_type())?;

                let struct_type = ctx
                    .type_converter()
                    .to_llvm_type(&field.struct_type, ctx.module())?;

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
            TypedAssignTarget::ArrayIndex(index) => {
                let array_ptr = ctx.get_ptr(&index.expr)?;
                let index_val = index.index.visit(ctx)?;
                let elem_type = index.typ.get_array_elem_type();
                let symbols = ctx.symbols();
                let array_type = ctx.type_converter().to_llvm_type(&index.typ, ctx.module())?;
                let zero = ctx.context().i32_type().const_zero();
                let elem_ptr = unsafe {
                    ctx.builder().build_in_bounds_gep(
                        array_type,
                        array_ptr,
                        &[zero, index_val.value()?.into_int_value()],
                        "array_assign_ptr",
                    )
                }
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to create array element pointer for assignment".to_string(),
                })?;
                Ok(LLVMVariable::new(elem_ptr, elem_type))
            }
            TypedAssignTarget::Deref(inner) => {
                // Evaluate the pointer expression, then load the pointer value stored in it.
                // The result is the address we write through.
                let ptr_holder = ctx.get_ptr(inner)?;
                let symbols = ctx.symbols();
                let pointee_type = match inner.get_type() {
                    crate::ast::Types::Pointer(t) => *t,
                    other => {
                        return Err(CodegenError::LLVMBuild {
                            message: format!("deref assign: expected pointer, got {other:?}"),
                        });
                    }
                };
                let llvm_ptr_type: inkwell::types::BasicTypeEnum =
                    ctx.type_converter().ptr_type().into();
                let loaded_ptr = ctx
                    .load(ptr_holder, llvm_ptr_type, "deref_assign_ptr")?
                    .into_pointer_value();
                Ok(LLVMVariable::new(loaded_ptr, pointee_type))
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
                let new_value = generate_add(context, current_value.value()?, value_val.value()?)?;
                context
                    .builder()
                    .build_store(var_ptr.value(), new_value)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;

                Ok(CodegenValue::new(
                    new_value,
                    value_val.unwrap_concrete()?.type_info().clone(),
                ))
            }
            Op::MinusEqual => {
                let new_value = generic_sub(context, current_value.value()?, value_val.value()?)?;
                context
                    .builder()
                    .build_store(var_ptr.value(), new_value)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;
                Ok(CodegenValue::new(
                    new_value,
                    value_val.unwrap_concrete()?.type_info().clone(),
                ))
            }
            Op::Assign => {
                context
                    .builder()
                    .build_store(var_ptr.value(), value_val.value()?)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to build store for assignment: {e:?}"),
                    })?;

                Ok(CodegenValue::new(
                    value_val.value()?,
                    value_val.unwrap_concrete()?.type_info().clone(),
                ))
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
            Self::ArrayIndex(index) => index.visit(context),
            Self::Deref(inner) => {
                // Read through the pointer: load the pointer, then load the pointee.
                let ptr_holder = context.get_ptr(inner)?;
                let llvm_ptr_type: inkwell::types::BasicTypeEnum =
                    context.type_converter().ptr_type().into();
                let loaded_ptr = context
                    .load(ptr_holder, llvm_ptr_type, "deref_read_ptr")?
                    .into_pointer_value();
                let pointee_type = match inner.get_type() {
                    crate::ast::Types::Pointer(t) => *t,
                    other => {
                        return Err(CodegenError::LLVMBuild {
                            message: format!("deref target read: expected pointer, got {other:?}"),
                        });
                    }
                };
                let symbols = context.symbols();
                let llvm_pointee = context
                    .type_converter()
                    .to_llvm_type(&pointee_type, context.module())?;
                context
                    .load(loaded_ptr, llvm_pointee, "deref_read_val")
                    .map(|val| CodegenValue::new(val, pointee_type))
            }
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
