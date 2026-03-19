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
            Self::Ident { ident, typ } => {
                let var_access = VariableAccess::new(ident, typ.visit_options());
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
            Self::MethodCall {
                mangled_name,
                receiver,
                args,
                typ,
            } => {
                // Get a pointer to the receiver (struct instance).
                // For an Ident receiver, use the variable pointer directly.
                // For other expressions, materialise the value and alloca it.
                let self_ptr = match receiver.as_ref() {
                    TypedExpr::Ident { ident, .. } => context.get_variable(ident)?.value(),
                    _ => {
                        let receiver_val = receiver.visit(context)?;
                        if receiver_val.value.is_pointer_value() {
                            receiver_val.value.into_pointer_value()
                        } else {
                            let compiler_context = context.symbols();
                            let struct_llvm_type = context
                                .type_converter()
                                .to_llvm_type(&receiver_val.type_info, compiler_context)?;
                            let temp_ptr = context
                                .builder()
                                .build_alloca(struct_llvm_type, "method_self_tmp")
                                .map_err(|e| CodegenError::LLVMBuild {
                                    message: format!("Failed to alloca method receiver: {e}"),
                                })?;
                            context
                                .builder()
                                .build_store(temp_ptr, receiver_val.value)
                                .map_err(|e| CodegenError::LLVMBuild {
                                    message: format!("Failed to store method receiver: {e}"),
                                })?;
                            temp_ptr
                        }
                    }
                };

                let function = context.get_function(mangled_name.inner())?;

                let mut arg_values: Vec<inkwell::values::BasicMetadataValueEnum> =
                    vec![self_ptr.into()];
                for arg in args {
                    let arg_val = arg.visit(context)?;
                    arg_values.push(arg_val.value.into());
                }

                // Look up the return type via the function signature
                let return_type = {
                    let name_ident = crate::tokens::Ident::new(
                        mangled_name.inner().to_string(),
                        Default::default(),
                    );
                    context
                        .symbols()
                        .get_function_signature(&name_ident)
                        .map_err(|_| CodegenError::FunctionNotFound {
                            name: mangled_name.inner().to_string(),
                        })?
                        .return_type()
                        .clone()
                };

                let call_result = context
                    .builder()
                    .build_call(function, &arg_values, "method_call")
                    .map_err(|_| CodegenError::LLVMBuild {
                        message: format!(
                            "Failed to generate method call to {}",
                            mangled_name.inner()
                        ),
                    })?;

                Ok(CodegenValue::new(
                    call_result.try_as_basic_value().unwrap_left(),
                    return_type,
                ))
            }
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
        let array_value = self.expr.visit(context)?;
        let array_ptr = array_value.value.into_pointer_value();

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

        let val = context
            .builder()
            .build_load(elem_type, elem_ptr, "array_elem")?;

        Ok(CodegenValue {
            value: val,
            type_info: self.typ.get_array_elem_type(),
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
        let compiler_context = context.symbols();

        let struct_ptr = match self.expr.as_ref() {
            crate::typed_ast::TypedExpr::Ident { ident, .. } => {
                context.get_variable(&ident)?.value()
            }
            _ => {
                let struct_value = self.expr.visit(context)?;
                if struct_value.value.is_pointer_value() {
                    struct_value.value.into_pointer_value()
                } else {
                    let struct_type = context
                        .type_converter()
                        .to_llvm_type(&self.struct_type, compiler_context)?;
                    let temp_ptr = context
                        .builder()
                        .build_alloca(struct_type, "temp_struct")
                        .map_err(|e| CodegenError::LLVMBuild {
                            message: format!("Failed to create temporary struct allocation: {e}"),
                        })?;
                    context
                        .builder()
                        .build_store(temp_ptr, struct_value.value)
                        .map_err(|e| CodegenError::LLVMBuild {
                            message: format!("Failed to store temporary struct: {e}"),
                        })?;
                    temp_ptr
                }
            }
        };

        let struct_type = context
            .type_converter()
            .to_llvm_type(&self.struct_type, compiler_context)?;

        let struct_name = self.struct_type.unwrap_struct_name();
        let strct = context.symbols().structs();
        let field_index = strct.field_index(struct_name, &self.field);

        let zero = context.context().i32_type().const_zero();
        let field_index = context
            .context()
            .i32_type()
            .const_int(field_index as u64, false);

        let field_val = unsafe {
            context.builder().build_in_bounds_gep(
                struct_type,
                struct_ptr,
                &[zero, field_index],
                "struct_fetch",
            )
        }
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create struct field pointer".to_string(),
        })?;

        let type_conv = context.type_converter();
        let field_llvm_type = type_conv.to_llvm_type(&self.field_type, compiler_context)?;

        let field_val = context
            .builder()
            .build_load(field_llvm_type, field_val, "field_access")?;

        Ok(CodegenValue {
            value: field_val,
            type_info: self.field_type.clone(),
        })
    }
}
