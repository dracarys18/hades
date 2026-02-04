use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::Op;
use crate::typed_ast::TypedExpr;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};

pub struct BinaryOp<'a> {
    pub left: &'a TypedExpr,
    pub op: &'a Op,
    pub right: &'a TypedExpr,
}

impl<'a> BinaryOp<'a> {
    pub fn new(left: &'a TypedExpr, op: &'a Op, right: &'a TypedExpr) -> Self {
        Self { left, op, right }
    }
}

impl<'a> Visit for BinaryOp<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let left_val = self.left.visit(context)?;
        let right_val = self.right.visit(context)?;

        let result_type = context
            .symbols()
            .infer_binary_type(&left_val.type_info, self.op, &right_val.type_info)
            .map_err(|_| CodegenError::TypeMismatch {
                expected: format!(
                    "{:?} {:?} {:?}",
                    left_val.type_info, self.op, right_val.type_info
                ),
                actual: "incompatible types".to_string(),
            })?;

        let result_val = match (&left_val.type_info, &right_val.type_info) {
            (Types::Int, Types::Int) => generate_int_binary_op(
                left_val.value.into_int_value(),
                self.op,
                right_val.value.into_int_value(),
                context,
            )?,
            (Types::Float, Types::Float) => generate_float_binary_op(
                left_val.value.into_float_value(),
                self.op,
                right_val.value.into_float_value(),
                context,
            )?,
            (Types::Bool, Types::Bool) => generate_bool_binary_op(
                left_val.value.into_int_value(),
                self.op,
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
