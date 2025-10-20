use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::ExprCodegen;
use crate::tokens::{Ident, Op};
use crate::typed_ast::{TypedExpr, TypedValue};
use indexmap::IndexMap;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};

impl ExprCodegen for TypedExpr {
    fn generate_expr<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        match self {
            Self::Value(value) => generate_value(value, context),
            Self::Ident { ident, .. } => generate_variable_access(ident, context),
            Self::Binary {
                left, op, right, ..
            } => generate_binary_op(left, op, right, context),
            Self::Unary { op, expr, .. } => generate_unary_op(op, expr, context),
            Self::Call { func, args, .. } => generate_function_call(func.inner(), args, context),
            Self::StructInit { name, fields, .. } => generate_struct_init(name, fields, context),
            _ => Err(CodegenError::LLVMBuild {
                message: format!("Expression type {:?} not implemented", self),
            }),
        }
    }
}

fn generate_value<'ctx>(
    value: &TypedValue,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    match value {
        TypedValue::Number(val) => {
            let llvm_val = context.context().i32_type().const_int(*val as u64, false);
            Ok(CodegenValue::new(llvm_val.into(), Types::Int))
        }
        TypedValue::Float(val) => {
            let llvm_val = context.context().f64_type().const_float(*val);
            Ok(CodegenValue::new(llvm_val.into(), Types::Float))
        }
        TypedValue::Boolean(val) => Ok(CodegenValue::bool_val(context.context(), *val)),
        TypedValue::String(val) => {
            let string_val = context.context().const_string(val.as_bytes(), true);
            Ok(CodegenValue::new(string_val.into(), Types::String))
        }
    }
}

fn generate_struct_init<'ctx>(
    name: &Ident,
    fields: &IndexMap<Ident, TypedExpr>,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let symbols = context.symbols();
    let struct_type = context
        .type_converter()
        .convert_struct_type(name, symbols)?;

    // Allocate the struct in memory (on the stack)
    let struct_ptr = context
        .builder()
        .build_alloca(struct_type, "struct_tmp")
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to alloca struct: {e}"),
        })?;

    // Fill its fields
    for (i, (_, field_expr)) in fields.iter().enumerate() {
        let field_val = field_expr.generate_expr(context)?;
        let field_ptr = context
            .builder()
            .build_struct_gep(struct_type, struct_ptr, i as u32, "field_ptr")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to get field ptr: {e}"),
            })?;

        context
            .builder()
            .build_store(field_ptr, field_val.value)
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to store field: {e}"),
            })?;
    }

    // Load the completed struct value
    let struct_value = context
        .builder()
        .build_load(struct_type, struct_ptr, "struct_val")
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to load struct: {e}"),
        })?;

    Ok(CodegenValue::new(struct_value, Types::Struct(name.clone())))
}

fn generate_variable_access<'ctx>(
    name: &Ident,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let var = context.get_variable(name)?;
    let var_ptr = var.value();
    let var_type = var.typ();
    let loaded_val = context.create_load(var_ptr, name.inner())?;

    Ok(CodegenValue::new(loaded_val, var_type.clone()))
}

fn generate_binary_op<'ctx>(
    left: &TypedExpr,
    op: &Op,
    right: &TypedExpr,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let left_val = left.generate_expr(context)?;
    let right_val = right.generate_expr(context)?;

    let result_type = context
        .symbols()
        .infer_binary_type(&left_val.type_info, op, &right_val.type_info)
        .map_err(|_| CodegenError::TypeMismatch {
            expected: format!(
                "{:?} {:?} {:?}",
                left_val.type_info, op, right_val.type_info
            ),
            actual: "incompatible types".to_string(),
        })?;

    let result_val = match (&left_val.type_info, &right_val.type_info) {
        (Types::Int, Types::Int) => generate_int_binary_op(
            left_val.value.into_int_value(),
            op,
            right_val.value.into_int_value(),
            context,
        )?,
        (Types::Float, Types::Float) => generate_float_binary_op(
            left_val.value.into_float_value(),
            op,
            right_val.value.into_float_value(),
            context,
        )?,
        (Types::Bool, Types::Bool) => generate_bool_binary_op(
            left_val.value.into_int_value(),
            op,
            right_val.value.into_int_value(),
            context,
        )?,
        _ => {
            return Err(CodegenError::TypeMismatch {
                expected: format!("{:?}", left_val.type_info),
                actual: format!("{:?}", right_val.type_info),
            });
        }
    };

    Ok(CodegenValue::new(result_val, result_type))
}

fn generate_int_binary_op<'ctx>(
    left: IntValue<'ctx>,
    op: &Op,
    right: IntValue<'ctx>,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let builder = context.builder();
    let result = match op {
        Op::Add | Op::Plus => {
            builder
                .build_int_add(left, right, "add")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int add failed: {:?}", e),
                })?
        }
        Op::Sub | Op::Minus => {
            builder
                .build_int_sub(left, right, "sub")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int sub failed: {:?}", e),
                })?
        }
        Op::Mul | Op::Multiply => {
            builder
                .build_int_mul(left, right, "mul")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int mul failed: {:?}", e),
                })?
        }
        Op::Div | Op::Divide => builder
            .build_int_signed_div(left, right, "div")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int div failed: {:?}", e),
            })?,
        Op::Mod => builder
            .build_int_signed_rem(left, right, "mod")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int mod failed: {:?}", e),
            })?,
        Op::Eq | Op::EqualEqual => builder
            .build_int_compare(IntPredicate::EQ, left, right, "eq")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int eq failed: {:?}", e),
            })?,
        Op::Ne | Op::BangEqual => builder
            .build_int_compare(IntPredicate::NE, left, right, "ne")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int ne failed: {:?}", e),
            })?,
        Op::Lt | Op::Less => builder
            .build_int_compare(IntPredicate::SLT, left, right, "lt")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int lt failed: {:?}", e),
            })?,
        Op::Le | Op::LessEqual => builder
            .build_int_compare(IntPredicate::SLE, left, right, "le")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int le failed: {:?}", e),
            })?,
        Op::Gt | Op::Greater => builder
            .build_int_compare(IntPredicate::SGT, left, right, "gt")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int gt failed: {:?}", e),
            })?,
        Op::Ge | Op::GreaterEqual => builder
            .build_int_compare(IntPredicate::SGE, left, right, "ge")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int ge failed: {:?}", e),
            })?,
        Op::BitAnd => {
            builder
                .build_and(left, right, "and")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int and failed: {:?}", e),
                })?
        }
        Op::BitOr => builder
            .build_or(left, right, "or")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int or failed: {:?}", e),
            })?,
        Op::BitXor => {
            builder
                .build_xor(left, right, "xor")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int xor failed: {:?}", e),
                })?
        }
        Op::Shl => {
            builder
                .build_left_shift(left, right, "shl")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int shl failed: {:?}", e),
                })?
        }
        Op::Shr => builder
            .build_right_shift(left, right, false, "shr")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Int shr failed: {:?}", e),
            })?,
        _ => {
            return Err(CodegenError::LLVMBuild {
                message: format!("Unsupported integer operation: {:?}", op),
            });
        }
    };

    Ok(result.into())
}

fn generate_float_binary_op<'ctx>(
    left: FloatValue<'ctx>,
    op: &Op,
    right: FloatValue<'ctx>,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let builder = context.builder();
    let result: BasicValueEnum = match op {
        Op::Add | Op::Plus | Op::PlusEqual => builder
            .build_float_add(left, right, "fadd")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float add failed: {:?}", e),
            })?
            .into(),
        Op::Sub | Op::Minus | Op::MinusEqual => builder
            .build_float_sub(left, right, "fsub")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float sub failed: {:?}", e),
            })?
            .into(),
        Op::Mul | Op::Multiply => builder
            .build_float_mul(left, right, "fmul")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float mul failed: {:?}", e),
            })?
            .into(),
        Op::Div | Op::Divide => builder
            .build_float_div(left, right, "fdiv")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float div failed: {:?}", e),
            })?
            .into(),
        Op::Mod => {
            let result = builder.build_float_rem(left, right, "frem").map_err(|e| {
                CodegenError::LLVMBuild {
                    message: format!("Float rem failed: {:?}", e),
                }
            })?;
            result.into()
        }
        Op::Eq | Op::EqualEqual => builder
            .build_float_compare(FloatPredicate::OEQ, left, right, "feq")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float eq failed: {:?}", e),
            })?
            .into(),
        Op::Ne | Op::BangEqual => builder
            .build_float_compare(FloatPredicate::ONE, left, right, "fne")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float ne failed: {:?}", e),
            })?
            .into(),
        Op::Lt | Op::Less => builder
            .build_float_compare(FloatPredicate::OLT, left, right, "flt")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float lt failed: {:?}", e),
            })?
            .into(),
        Op::Le | Op::LessEqual => builder
            .build_float_compare(FloatPredicate::OLE, left, right, "fle")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float le failed: {:?}", e),
            })?
            .into(),
        Op::Gt | Op::Greater => builder
            .build_float_compare(FloatPredicate::OGT, left, right, "fgt")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float gt failed: {:?}", e),
            })?
            .into(),
        Op::Ge | Op::GreaterEqual => builder
            .build_float_compare(FloatPredicate::OGE, left, right, "fge")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float ge failed: {:?}", e),
            })?
            .into(),
        _ => {
            return Err(CodegenError::LLVMBuild {
                message: format!("Unsupported float operation: {:?}", op),
            });
        }
    };

    Ok(result.into())
}

fn generate_bool_binary_op<'ctx>(
    left: IntValue<'ctx>,
    op: &Op,
    right: IntValue<'ctx>,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let builder = context.builder();
    let result = match op {
        Op::And | Op::BoleanAnd => {
            builder
                .build_and(left, right, "and")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Bool and failed: {:?}", e),
                })?
        }
        Op::Or | Op::BooleanOr => {
            builder
                .build_or(left, right, "or")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Bool or failed: {:?}", e),
                })?
        }
        Op::Eq | Op::EqualEqual => builder
            .build_int_compare(IntPredicate::EQ, left, right, "eq")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Bool eq failed: {:?}", e),
            })?,
        Op::Ne | Op::BangEqual => builder
            .build_int_compare(IntPredicate::NE, left, right, "ne")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Bool ne failed: {:?}", e),
            })?,
        _ => {
            return Err(CodegenError::LLVMBuild {
                message: format!("Unsupported boolean operation: {:?}", op),
            });
        }
    };

    Ok(result.into())
}

fn generate_unary_op<'ctx>(
    op: &Op,
    operand: &TypedExpr,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let operand_val = operand.generate_expr(context)?;

    let result_type = context
        .symbols()
        .infer_unary_type(op, &operand_val.type_info)
        .map_err(|_| CodegenError::TypeMismatch {
            expected: format!("{:?} {:?}", op, operand_val.type_info),
            actual: "incompatible type".to_string(),
        })?;

    let builder = context.builder();
    let result_val: BasicValueEnum = match (&operand_val.type_info, op) {
        (Types::Int, Op::Minus | Op::Sub) => {
            let zero = context.context().i32_type().const_zero();
            builder
                .build_int_sub(zero, operand_val.value.into_int_value(), "neg")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int neg failed: {:?}", e),
                })?
                .into()
        }
        (Types::Float, Op::Minus | Op::Sub) => builder
            .build_float_neg(operand_val.value.into_float_value(), "fneg")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Float neg failed: {:?}", e),
            })?
            .into(),
        (Types::Bool, Op::Not) => {
            let true_val = context.context().bool_type().const_all_ones();
            builder
                .build_xor(operand_val.value.into_int_value(), true_val, "not")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Bool not failed: {:?}", e),
                })?
                .into()
        }
        (Types::Int, Op::BitNot) => {
            let all_ones = operand_val
                .value
                .into_int_value()
                .get_type()
                .const_all_ones();
            builder
                .build_xor(operand_val.value.into_int_value(), all_ones, "bitnot")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Int bitnot failed: {:?}", e),
                })?
                .into()
        }
        _ => {
            return Err(CodegenError::LLVMBuild {
                message: format!(
                    "Unsupported unary operation: {:?} {:?}",
                    op, operand_val.type_info
                ),
            });
        }
    };

    Ok(CodegenValue::new(result_val, result_type))
}

fn generate_function_call<'ctx>(
    name: &str,
    args: &[TypedExpr],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let function = context.get_function(name)?;

    let mut arg_values = Vec::new();
    for arg in args {
        let arg_val = arg.generate_expr(context)?;
        arg_values.push(arg_val.value.into());
    }

    let call_result = context
        .builder()
        .build_call(function, &arg_values, "call")
        .map_err(|_| CodegenError::LLVMBuild {
            message: format!("Failed to generate function call to {}", name),
        })?;

    let name_ident = crate::tokens::Ident::new(name.to_string(), Default::default());
    let return_type = context
        .symbols()
        .get_function_signature(&name_ident)
        .map_err(|_| CodegenError::FunctionNotFound {
            name: name.to_string(),
        })?
        .return_type()
        .clone();

    Ok(CodegenValue::new(
        call_result.try_as_basic_value().unwrap_left(),
        return_type,
    ))
}

// fn generate_array_access<'ctx>(
//     _array: &TypedExpr,
//     _index: &TypedExpr,
//     _context: &mut LLVMContext<'ctx>,
// ) -> CodegenResult<CodegenValue<'ctx>> {
//     Err(CodegenError::LLVMBuild {
//         message: "Array access not implemented yet".to_string(),
//     })
// }

// fn generate_field_access<'ctx>(
//     _object: &TypedExpr,
//     _field: &str,
//     _context: &mut LLVMContext<'ctx>,
// ) -> CodegenResult<CodegenValue<'ctx>> {
//     Err(CodegenError::LLVMBuild {
//         message: "Field access not implemented yet".to_string(),
//     })
// }
