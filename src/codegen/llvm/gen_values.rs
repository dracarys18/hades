use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::ValueCodegen;
use crate::typed_ast::TypedValue;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

impl ValueCodegen for TypedValue {
    fn generate_value<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        match self {
            TypedValue::Number(val) => generate_int_value(*val, context),
            TypedValue::Float(val) => generate_float_value(*val, context),
            TypedValue::Boolean(val) => generate_bool_value(*val, context),
            TypedValue::String(val) => generate_string_value(val, context),
        }
    }
}

fn generate_bool_value<'ctx>(
    val: bool,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().bool_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Bool))
}

fn generate_string_value<'ctx>(
    val: &str,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let string_val = context.context().const_string(val.as_bytes(), true);
    let global_string = context
        .module()
        .add_global(string_val.get_type(), None, "str");
    global_string.set_initializer(&string_val);
    global_string.set_constant(true);

    let string_ptr = global_string.as_pointer_value();
    let zero = context.context().i32_type().const_zero();
    let gep = unsafe {
        context
            .builder()
            .build_in_bounds_gep(string_val.get_type(), string_ptr, &[zero, zero], "str_ptr")
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to create string pointer".to_string(),
            })?
    };

    Ok(CodegenValue::new(gep.into(), Types::String))
}

fn generate_int_value<'ctx>(
    val: i64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().i32_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Int))
}

fn generate_float_value<'ctx>(
    val: f64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().f64_type().const_float(val);
    Ok(CodegenValue::new(llvm_val.into(), Types::Float))
}

pub fn cast_value<'ctx>(
    value: CodegenValue<'ctx>,
    target_type: &Types,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    if &value.type_info == target_type {
        return Ok(value);
    }

    let builder = context.builder();
    let result_val = match (&value.type_info, target_type) {
        (Types::Int, Types::Float) => {
            let int_val = value.value.into_int_value();
            builder
                .build_signed_int_to_float(int_val, context.context().f64_type(), "sitofp")
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to convert int to float".to_string(),
                })?
                .into()
        }
        (Types::Float, Types::Int) => {
            let float_val = value.value.into_float_value();
            builder
                .build_float_to_signed_int(float_val, context.context().i32_type(), "fptosi")
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to convert float to int".to_string(),
                })?
                .into()
        }
        _ => {
            return Err(CodegenError::TypeConversion {
                from: format!("{:?}", value.type_info),
                to: format!("{:?}", target_type),
            });
        }
    };

    Ok(CodegenValue::new(result_val, target_type.clone()))
}

pub fn create_zero_value<'ctx>(
    ty: &Types,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let zero_val = match ty {
        Types::Int => context.context().i32_type().const_zero().into(),
        Types::Float => context.context().f64_type().const_zero().into(),
        Types::Bool => context.context().bool_type().const_zero().into(),
        Types::String => {
            let empty_str = context.context().const_string(&[], true);
            empty_str.into()
        }
        _ => {
            return Err(CodegenError::TypeConversion {
                from: "zero".to_string(),
                to: format!("{:?}", ty),
            });
        }
    };

    Ok(CodegenValue::new(zero_val, ty.clone()))
}

pub fn create_one_value<'ctx>(
    ty: &Types,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let one_val = match ty {
        Types::Int => context.context().i32_type().const_int(1, false).into(),
        Types::Float => context.context().f64_type().const_float(1.0).into(),
        Types::Bool => context.context().bool_type().const_int(1, false).into(),
        _ => {
            return Err(CodegenError::TypeConversion {
                from: "one".to_string(),
                to: format!("{:?}", ty),
            });
        }
    };

    Ok(CodegenValue::new(one_val, ty.clone()))
}

pub fn compare_values<'ctx>(
    left: &CodegenValue<'ctx>,
    right: &CodegenValue<'ctx>,
    predicate: ComparisonPredicate,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    if !context
        .type_converter()
        .are_compatible(&left.type_info, &right.type_info)
    {
        return Err(CodegenError::TypeMismatch {
            expected: format!("{:?}", left.type_info),
            actual: format!("{:?}", right.type_info),
        });
    }

    let result_val = match (&left.type_info, &right.type_info) {
        (Types::Int, Types::Int) => {
            let int_pred = match predicate {
                ComparisonPredicate::Equal => IntPredicate::EQ,
                ComparisonPredicate::NotEqual => IntPredicate::NE,
                ComparisonPredicate::LessThan => IntPredicate::SLT,
                ComparisonPredicate::LessEqual => IntPredicate::SLE,
                ComparisonPredicate::GreaterThan => IntPredicate::SGT,
                ComparisonPredicate::GreaterEqual => IntPredicate::SGE,
            };
            context
                .builder()
                .build_int_compare(
                    int_pred,
                    left.value.into_int_value(),
                    right.value.into_int_value(),
                    "cmp",
                )
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to compare integers".to_string(),
                })?
                .into()
        }
        (Types::Float, Types::Float) => {
            let float_pred = match predicate {
                ComparisonPredicate::Equal => FloatPredicate::OEQ,
                ComparisonPredicate::NotEqual => FloatPredicate::ONE,
                ComparisonPredicate::LessThan => FloatPredicate::OLT,
                ComparisonPredicate::LessEqual => FloatPredicate::OLE,
                ComparisonPredicate::GreaterThan => FloatPredicate::OGT,
                ComparisonPredicate::GreaterEqual => FloatPredicate::OGE,
            };
            context
                .builder()
                .build_float_compare(
                    float_pred,
                    left.value.into_float_value(),
                    right.value.into_float_value(),
                    "fcmp",
                )
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to compare floats".to_string(),
                })?
                .into()
        }
        _ => {
            return Err(CodegenError::TypeMismatch {
                expected: format!("{:?}", left.type_info),
                actual: format!("{:?}", right.type_info),
            });
        }
    };

    Ok(CodegenValue::new(result_val, Types::Bool))
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonPredicate {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}
