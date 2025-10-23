use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::Op;
use crate::typed_ast::TypedExpr;
use inkwell::values::BasicValueEnum;

pub struct UnaryOp<'a> {
    pub op: &'a Op,
    pub operand: &'a TypedExpr,
}

impl<'a> UnaryOp<'a> {
    pub fn new(op: &'a Op, operand: &'a TypedExpr) -> Self {
        Self { op, operand }
    }
}

impl<'a> Visit for UnaryOp<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let operand_val = self.operand.visit(context)?;

        let result_type = context
            .symbols()
            .infer_unary_type(self.op, &operand_val.type_info)
            .map_err(|_| CodegenError::TypeMismatch {
                expected: format!("{:?} {:?}", self.op, operand_val.type_info),
                actual: "incompatible type".to_string(),
            })?;

        let builder = context.builder();
        let result_val: BasicValueEnum = match (&operand_val.type_info, self.op) {
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
                        self.op, operand_val.type_info
                    ),
                });
            }
        };

        Ok(CodegenValue::new(result_val, result_type))
    }
}
