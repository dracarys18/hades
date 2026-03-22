use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::{
    TypedArrayIndex, TypedAssignExpr, TypedBinaryExpr, TypedExpr, TypedFieldAccess,
};
use inkwell::values::PointerValue;

pub mod assign;
pub mod binary;
pub mod call;
pub mod struct_init;
pub mod unary;
pub mod variable;

pub use assign::Assignment;
pub use binary::BinaryOp;
pub use call::{visit_function_call, visit_method_call};
pub use struct_init::StructInit;
pub use unary::UnaryOp;
pub use variable::VariableAccess;

pub(super) fn get_ptr<'ctx>(
    expr: &TypedExpr,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<PointerValue<'ctx>> {
    if let TypedExpr::Ident { ident, .. } = expr {
        return context.get_variable(ident).map(|v| v.value());
    }
    expr.visit(context).and_then(|val| {
        val.value.try_into().or_else(|_| {
            let symbols = context.symbols();
            context
                .type_converter()
                .to_llvm_type(&val.type_info, symbols)
                .and_then(|t| {
                    context.builder().build_alloca(t, "tmp_ptr").map_err(|e| {
                        CodegenError::LLVMBuild {
                            message: e.to_string(),
                        }
                    })
                })
                .and_then(|ptr| {
                    context
                        .builder()
                        .build_store(ptr, val.value)
                        .map_err(|e| CodegenError::LLVMBuild {
                            message: e.to_string(),
                        })
                        .map(|_| ptr)
                })
        })
    })
}

impl Visit for TypedExpr {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Self::Value(value) => value.visit(context),
            Self::Ident { ident, typ } => {
                VariableAccess::new(ident, typ.visit_options()).visit(context)
            }
            Self::Binary(binary) => binary.visit(context),
            Self::Unary { op, expr, .. } => UnaryOp::new(op, expr).visit(context),
            Self::Call {
                func,
                args,
                receiver: Some(recv),
                ..
            } => visit_method_call(func.inner(), recv, args, context),
            Self::Call {
                func,
                args,
                receiver: None,
                ..
            } => visit_function_call(func.inner(), args, context),
            Self::StructInit { name, fields, .. } => StructInit::new(name, fields).visit(context),
            Self::Assign(assign) => assign.visit(context),
            Self::FieldAccess(field) => field.visit(context),
            Self::ArrayIndex(index) => index.visit(context),
        }
    }
}

impl Visit for TypedAssignExpr {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        Assignment::new(&self.target, &self.op, &self.value).visit(context)
    }
}

impl Visit for TypedArrayIndex {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let array_ptr = get_ptr(&self.expr, context)?;
        let index_value = self.index.visit(context)?;
        let symbols = context.symbols();
        let elem_type = context
            .type_converter()
            .to_llvm_type(&self.typ.get_array_elem_type(), symbols)?;
        let array_type = context.type_converter().to_llvm_type(&self.typ, symbols)?;

        let zero = context.context().i32_type().const_zero();
        let elem_ptr = unsafe {
            context.builder().build_in_bounds_gep(
                array_type,
                array_ptr,
                &[zero, index_value.value.into_int_value()],
                "array_elem_ptr",
            )?
        };

        context
            .builder()
            .build_load(elem_type, elem_ptr, "array_elem")
            .map(|val| CodegenValue {
                value: val,
                type_info: self.typ.get_array_elem_type(),
            })
            .map_err(CodegenError::from)
    }
}

impl Visit for TypedBinaryExpr {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        BinaryOp::new(&self.left, &self.op, &self.right).visit(context)
    }
}

impl Visit for TypedFieldAccess {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let struct_ptr = get_ptr(self.expr.as_ref(), context)?;
        let compiler_context = context.symbols();

        let struct_type = context
            .type_converter()
            .to_llvm_type(&self.struct_type, compiler_context)?;

        let struct_name = self.struct_type.unwrap_struct_name();
        let field_index = context
            .symbols()
            .structs()
            .field_index(struct_name, &self.field);

        let zero = context.context().i32_type().const_zero();
        let field_index_val = context
            .context()
            .i32_type()
            .const_int(field_index as u64, false);

        let field_ptr = unsafe {
            context.builder().build_in_bounds_gep(
                struct_type,
                struct_ptr,
                &[zero, field_index_val],
                "struct_fetch",
            )
        }
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create struct field pointer".to_string(),
        })?;

        let field_llvm_type = context
            .type_converter()
            .to_llvm_type(&self.field_type, compiler_context)?;

        context
            .builder()
            .build_load(field_llvm_type, field_ptr, "field_access")
            .map(|val| CodegenValue {
                value: val,
                type_info: self.field_type.clone(),
            })
            .map_err(CodegenError::from)
    }
}
