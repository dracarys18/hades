use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::VisitOptions;
use crate::tokens::Op;
use crate::typed_ast::{
    TypedArrayIndex, TypedAssignExpr, TypedBinaryExpr, TypedExpr, TypedFieldAccess,
};
use inkwell::values::PointerValue;
use inkwell::AddressSpace;

pub mod asexpr;
pub mod assign;
pub mod binary;
pub mod call;
pub mod struct_init;
pub mod unary;
pub mod variable;

pub use assign::Assignment;
pub use binary::BinaryOp;
pub use call::{FunctionCall, MethodCall};
pub use struct_init::StructInit;
pub use unary::UnaryOp;
pub use variable::VariableAccess;

impl<'ctx> LLVMContext<'ctx> {
    pub(crate) fn deref_if_pointer(
        &mut self,
        raw_ptr: PointerValue<'ctx>,
        expr_type: &Types,
    ) -> CodegenResult<PointerValue<'ctx>> {
        if let Types::Pointer(_) = expr_type {
            self.load(
                raw_ptr,
                self.context().ptr_type(AddressSpace::default()).into(),
                "deref_for_field",
            )
            .map(|v| v.into_pointer_value())
        } else {
            Ok(raw_ptr)
        }
    }

    pub(super) fn get_ptr(&mut self, expr: &TypedExpr) -> CodegenResult<PointerValue<'ctx>> {
        if let TypedExpr::Ident { ident, .. } = expr {
            return self.get_variable(ident).map(|v| v.value());
        }
        if let TypedExpr::Unary {
            op: Op::Deref,
            expr: inner,
            ..
        } = expr
        {
            let ptr_val = inner.visit(self)?;
            return ptr_val
                .value()?
                .try_into()
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "deref get_ptr: expected pointer value".to_string(),
                });
        }
        if let TypedExpr::FieldAccess(field) = expr {
            let raw_ptr = self.get_ptr(field.expr.as_ref())?;
            let struct_ptr = self.deref_if_pointer(raw_ptr, &field.expr.get_type())?;
            let symbols = self.symbols();
            let struct_type = self
                .type_converter()
                .to_llvm_type(&field.struct_type, symbols)?;
            let struct_name = field.struct_type.unwrap_struct_name();
            let field_index = self
                .symbols()
                .structs()
                .field_index(struct_name, &field.field);
            let zero = self.context().i32_type().const_zero();
            let field_index_val = self
                .context()
                .i32_type()
                .const_int(field_index as u64, false);
            return unsafe {
                self.builder().build_in_bounds_gep(
                    struct_type,
                    struct_ptr,
                    &[zero, field_index_val],
                    "field_lval_ptr",
                )
            }
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to create struct field lval pointer".to_string(),
            });
        }
        if let TypedExpr::ArrayIndex(index) = expr {
            let array_ptr = self.get_ptr(&index.expr)?;
            let index_value = index.index.visit(self)?;
            let symbols = self.symbols();
            let array_type = self.type_converter().to_llvm_type(&index.typ, symbols)?;
            let zero = self.context().i32_type().const_zero();
            return unsafe {
                self.builder().build_in_bounds_gep(
                    array_type,
                    array_ptr,
                    &[zero, index_value.value()?.into_int_value()],
                    "array_elem_ptr",
                )
            }
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to create array element lval pointer".to_string(),
            });
        }
        let val = expr.visit(self)?;
        if let Ok(ptr) = val.value()?.try_into() {
            return Ok(ptr);
        }
        let type_info = val.unwrap_concrete()?.type_info();
        let symbols = self.symbols();
        let t = self.type_converter().to_llvm_type(&type_info, symbols)?;
        let ptr = self.create_alloca("tmp_ptr", t)?;
        self.builder()
            .build_store(ptr, val.value()?)
            .map_err(|e| CodegenError::LLVMBuild {
                message: e.to_string(),
            })?;
        Ok(ptr)
    }
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
            } => MethodCall {
                name: func.inner(),
                receiver: recv,
                args,
            }
            .visit(context),
            Self::Call {
                func,
                args,
                receiver: None,
                ..
            } => FunctionCall {
                name: func.inner(),
                args,
            }
            .visit(context),
            Self::StructInit {
                name,
                fields,
                is_const,
                ..
            } => StructInit::new(name, fields, *is_const).visit(context),
            Self::Assign(assign) => assign.visit(context),
            Self::FieldAccess(field) => field.visit(context),
            Self::ArrayIndex(index) => index.visit(context),
            Self::Null(typ) => {
                let ptr = context
                    .context()
                    .ptr_type(AddressSpace::default())
                    .const_null();
                Ok(CodegenValue::new(ptr.into(), typ.clone()))
            }
            Self::As(as_expr) => as_expr.visit(context),
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
        let array_ptr = context.get_ptr(&self.expr)?;
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
                &[zero, index_value.value()?.into_int_value()],
                "array_elem_ptr",
            )?
        };

        context
            .load(elem_ptr, elem_type, "array_elem")
            .map(|val| CodegenValue::new(val, self.typ.get_array_elem_type()))
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
        let raw_ptr = context.get_ptr(self.expr.as_ref())?;
        let struct_ptr = context.deref_if_pointer(raw_ptr, &self.expr.get_type())?;

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
            .load(field_ptr, field_llvm_type, "field_access")
            .map(|val| CodegenValue::new(val, self.field_type.clone()))
    }
}
