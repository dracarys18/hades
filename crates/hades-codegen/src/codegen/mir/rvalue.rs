use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use hades_ast::Types;
use hades_mir::mir::rvalue::Rvalue;
use hades_tokens::Op;
use inkwell::values::BasicValueEnum;

impl Visit for Rvalue {
    type Output<'ctx> = (BasicValueEnum<'ctx>, Types);

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Rvalue::Use(operand) => operand.visit(context),
            Rvalue::BinaryOp(op, left, right) => {
                let (lhs, lhs_ty) = left.visit(context)?;
                let (rhs, rhs_ty) = right.visit(context)?;
                let value = context.build_binary_op(&lhs_ty, op, lhs, &rhs_ty, rhs)?;
                Ok((value, binary_result_type(&lhs_ty, op)))
            }
            Rvalue::UnaryOp(op, operand) => {
                let (value, ty) = operand.visit(context)?;
                let result = context.build_unary_op(op, &ty, value)?;
                Ok((result, unary_result_type(op, &ty)))
            }
            Rvalue::Cast(operand, target) => {
                let (value, source_ty) = operand.visit(context)?;
                let casted = context.build_cast(value, &source_ty, target)?;
                Ok((casted, target.clone()))
            }
            Rvalue::Aggregate(_, _) | Rvalue::Repeat(_, _) => unreachable!(
                "aggregate rvalues are stored directly into their place, not evaluated to a value"
            ),
        }
    }
}

fn binary_result_type(left_ty: &Types, op: &Op) -> Types {
    match op {
        Op::Eq
        | Op::EqualEqual
        | Op::Ne
        | Op::BangEqual
        | Op::Lt
        | Op::Less
        | Op::Le
        | Op::LessEqual
        | Op::Gt
        | Op::Greater
        | Op::Ge
        | Op::GreaterEqual
        | Op::And
        | Op::BoleanAnd
        | Op::Or
        | Op::BooleanOr => Types::Bool,
        _ => left_ty.clone(),
    }
}

fn unary_result_type(op: &Op, operand_ty: &Types) -> Types {
    match op {
        Op::Deref => match operand_ty {
            Types::Pointer(inner) => (**inner).clone(),
            other => other.clone(),
        },
        _ => operand_ty.clone(),
    }
}
