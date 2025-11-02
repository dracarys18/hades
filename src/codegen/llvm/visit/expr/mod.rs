use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::{
    TypedArrayIndex, TypedAssignExpr, TypedBinaryExpr, TypedExpr, TypedFieldAccess,
};

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
            Self::Binary(binary) => binary.visit(context),
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
            Self::Assign(assign) => assign.visit(context),
            Self::FieldAccess(field) => field.visit(context),
            Self::ArrayIndex(index) => index.visit(context),
        }
    }
}

impl Visit for TypedAssignExpr {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let assignment = Assignment::new(&self.target, &self.op, &self.value);
        assignment.visit(context)
    }
}

impl Visit for TypedArrayIndex {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let array_value = context.get_variable(&self.name)?;
        let index_value = self.index.visit(context)?;

        let symbols = context.symbols();
        let type_conv = context.type_converter();

        let llvm_array_type = type_conv.to_llvm_type(&self.typ, symbols)?;
        let llvm_val_type = type_conv.to_llvm_type(&self.typ.get_array_elem_type(), symbols)?;

        let llvm_value = unsafe {
            context.builder().build_in_bounds_gep(
                llvm_array_type,
                array_value.value(),
                &[
                    context.context().i64_type().const_zero(),
                    index_value.value.into_int_value(),
                ],
                "array_idx_access",
            )
        }
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to build GEP for array index access: {e}"),
        })?;

        let val = context
            .builder()
            .build_load(llvm_val_type, llvm_value, "array_idx_load")?;

        Ok(CodegenValue {
            value: val,
            type_info: self.typ.clone(),
        })
    }
}

impl Visit for TypedBinaryExpr {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let binary_op = BinaryOp::new(&self.left, &self.op, &self.right);
        binary_op.visit(context)
    }
}

impl Visit for TypedFieldAccess {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let struct_val = VariableAccess::new(&self.name)
            .visit(context)?
            .value
            .into_struct_value();

        let struct_name = self.struct_type.unwrap_struct_name();
        let strct = context.symbols().structs();
        let field_index = strct.field_index(struct_name, &self.field);
        let field_val =
            struct_val
                .get_field_at_index(field_index as u32)
                .ok_or(CodegenError::LLVMBuild {
                    message: format!(
                        "Failed to get field '{}' at index {} from struct '{}'",
                        self.field.inner(),
                        field_index,
                        self.name.inner()
                    ),
                })?;

        let compiler_context = context.symbols();
        let type_conv = context.type_converter();
        let field_llvm_type = type_conv.to_llvm_type(&self.field_type, compiler_context)?;

        let field_val = context.builder().build_load(
            field_llvm_type,
            field_val.into_pointer_value(),
            "field_access",
        )?;

        Ok(CodegenValue {
            value: field_val,
            type_info: self.field_type.clone(),
        })
    }
}
