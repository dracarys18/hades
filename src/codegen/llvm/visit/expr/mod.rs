use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedExpr;

pub mod assign;
pub mod binary;
pub mod call;
pub mod struct_init;
pub mod unary;
pub mod variable;

pub use assign::Assignment;
pub use binary::BinaryOp;
pub use call::FunctionCall;
pub use struct_init::StructInit;
pub use unary::UnaryOp;
pub use variable::VariableAccess;

impl Visit for TypedExpr {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Self::Value(value) => value.visit(context),
            Self::Ident { ident, .. } => {
                let var_access = VariableAccess::new(ident);
                var_access.visit(context)
            }
            Self::Binary {
                left, op, right, ..
            } => {
                let binary_op = BinaryOp::new(left, op, right);
                binary_op.visit(context)
            }
            Self::Unary { op, expr, .. } => {
                let unary_op = UnaryOp::new(op, expr);
                unary_op.visit(context)
            }
            Self::Call { func, args, .. } => {
                let function_call = FunctionCall::new(func.inner(), args);
                function_call.visit(context)
            }
            Self::StructInit { name, fields, .. } => {
                let struct_init = StructInit::new(name, fields);
                struct_init.visit(context)
            }
            Self::Assign {
                name, value, op, ..
            } => {
                let assignment = Assignment::new(name, op, value);
                assignment.visit(context)
            }
            _ => Err(CodegenError::LLVMBuild {
                message: format!("Expression type {:?} not implemented", self),
            }),
        }
    }
}
