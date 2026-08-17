use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use hades_ast::Types;
use hades_tokens::{Name, Op};
use inkwell::AddressSpace;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, IntValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

impl<'ctx> LLVMContext<'ctx> {
    pub fn deref_if_pointer(
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

    pub fn build_call(
        &mut self,
        name: &str,
        arg_values: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<CodegenValue<'ctx>> {
        let name_fn = Name::new(name.to_string(), Default::default());
        let sig = self
            .symbols()
            .get_function_signature(&name_fn)
            .map_err(|_| CodegenError::FunctionNotFound {
                name: name.to_string(),
            })?
            .clone();

        let function = self.get_function(name, &sig)?;
        let call_site = self
            .builder()
            .build_call(function, arg_values, "call")
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to generate function call to {name}"),
            })?;

        Ok(match call_site.try_as_basic_value().basic() {
            Some(v) => CodegenValue::new(v, sig.return_type().clone()),
            None => CodegenValue::Void,
        })
    }

    pub fn build_binary_op(
        &mut self,
        left_ty: &Types,
        op: &Op,
        left: BasicValueEnum<'ctx>,
        right_ty: &Types,
        right: BasicValueEnum<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        match (left_ty, right_ty) {
            (Types::Int, Types::Int) => {
                self.build_int_binary_op(left.into_int_value(), op, right.into_int_value())
            }
            (Types::Char, Types::Char) => {
                self.build_int_binary_op(left.into_int_value(), op, right.into_int_value())
            }
            (Types::Float, Types::Float) => {
                self.build_float_binary_op(left.into_float_value(), op, right.into_float_value())
            }
            (Types::Bool, Types::Bool) => {
                self.build_bool_binary_op(left.into_int_value(), op, right.into_int_value())
            }
            (Types::Pointer(_), Types::Pointer(_)) => self.build_pointer_compare(
                left.into_pointer_value(),
                op,
                right.into_pointer_value(),
            ),
            _ => Err(CodegenError::TypeMismatch {
                expected: format!("{left_ty:?}"),
                actual: format!("{right_ty:?}"),
            }),
        }
    }

    fn build_int_binary_op(
        &mut self,
        left: IntValue<'ctx>,
        op: &Op,
        right: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let builder = self.builder();
        let result = match op {
            Op::Add | Op::Plus => builder.build_int_add(left, right, "add"),
            Op::Sub | Op::Minus => builder.build_int_sub(left, right, "sub"),
            Op::Mul | Op::Multiply => builder.build_int_mul(left, right, "mul"),
            Op::Div | Op::Divide => builder.build_int_signed_div(left, right, "div"),
            Op::Mod => builder.build_int_signed_rem(left, right, "mod"),
            Op::Eq | Op::EqualEqual => {
                builder.build_int_compare(IntPredicate::EQ, left, right, "eq")
            }
            Op::Ne | Op::BangEqual => {
                builder.build_int_compare(IntPredicate::NE, left, right, "ne")
            }
            Op::Lt | Op::Less => builder.build_int_compare(IntPredicate::SLT, left, right, "lt"),
            Op::Le | Op::LessEqual => {
                builder.build_int_compare(IntPredicate::SLE, left, right, "le")
            }
            Op::Gt | Op::Greater => builder.build_int_compare(IntPredicate::SGT, left, right, "gt"),
            Op::Ge | Op::GreaterEqual => {
                builder.build_int_compare(IntPredicate::SGE, left, right, "ge")
            }
            Op::BitAnd => builder.build_and(left, right, "and"),
            Op::BitOr => builder.build_or(left, right, "or"),
            Op::BitXor => builder.build_xor(left, right, "xor"),
            Op::Shl => builder.build_left_shift(left, right, "shl"),
            Op::Shr => builder.build_right_shift(left, right, false, "shr"),
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Unsupported integer operation: {op:?}"),
                });
            }
        };
        Ok(result.map_err(CodegenError::from)?.into())
    }

    fn build_float_binary_op(
        &mut self,
        left: FloatValue<'ctx>,
        op: &Op,
        right: FloatValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let builder = self.builder();
        let result = match op {
            Op::Add | Op::Plus | Op::PlusEqual => {
                builder.build_float_add(left, right, "fadd").map(Into::into)
            }
            Op::Sub | Op::Minus | Op::MinusEqual => {
                builder.build_float_sub(left, right, "fsub").map(Into::into)
            }
            Op::Mul | Op::Multiply => builder.build_float_mul(left, right, "fmul").map(Into::into),
            Op::Div | Op::Divide => builder.build_float_div(left, right, "fdiv").map(Into::into),
            Op::Mod => builder.build_float_rem(left, right, "frem").map(Into::into),
            Op::Eq | Op::EqualEqual => builder
                .build_float_compare(FloatPredicate::OEQ, left, right, "feq")
                .map(Into::into),
            Op::Ne | Op::BangEqual => builder
                .build_float_compare(FloatPredicate::ONE, left, right, "fne")
                .map(Into::into),
            Op::Lt | Op::Less => builder
                .build_float_compare(FloatPredicate::OLT, left, right, "flt")
                .map(Into::into),
            Op::Le | Op::LessEqual => builder
                .build_float_compare(FloatPredicate::OLE, left, right, "fle")
                .map(Into::into),
            Op::Gt | Op::Greater => builder
                .build_float_compare(FloatPredicate::OGT, left, right, "fgt")
                .map(Into::into),
            Op::Ge | Op::GreaterEqual => builder
                .build_float_compare(FloatPredicate::OGE, left, right, "fge")
                .map(Into::into),
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Unsupported float operation: {op:?}"),
                });
            }
        };
        result.map_err(CodegenError::from)
    }

    fn build_bool_binary_op(
        &mut self,
        left: IntValue<'ctx>,
        op: &Op,
        right: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let builder = self.builder();
        let result = match op {
            Op::And | Op::BoleanAnd => builder.build_and(left, right, "and"),
            Op::Or | Op::BooleanOr => builder.build_or(left, right, "or"),
            Op::Eq | Op::EqualEqual => {
                builder.build_int_compare(IntPredicate::EQ, left, right, "eq")
            }
            Op::Ne | Op::BangEqual => {
                builder.build_int_compare(IntPredicate::NE, left, right, "ne")
            }
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Unsupported boolean operation: {op:?}"),
                });
            }
        };
        Ok(result.map_err(CodegenError::from)?.into())
    }

    fn build_pointer_compare(
        &mut self,
        left: PointerValue<'ctx>,
        op: &Op,
        right: PointerValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let i64_type = self.context().i64_type();
        let lhs_int = self
            .builder()
            .build_ptr_to_int(left, i64_type, "ptr_to_int_l")
            .map_err(CodegenError::from)?;
        let rhs_int = self
            .builder()
            .build_ptr_to_int(right, i64_type, "ptr_to_int_r")
            .map_err(CodegenError::from)?;
        let pred = match op {
            Op::Eq | Op::EqualEqual => IntPredicate::EQ,
            Op::Ne | Op::BangEqual => IntPredicate::NE,
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Unsupported pointer comparison op: {op:?}"),
                });
            }
        };
        Ok(self
            .builder()
            .build_int_compare(pred, lhs_int, rhs_int, "ptrcmp")
            .map_err(CodegenError::from)?
            .into())
    }

    pub fn build_unary_op(
        &mut self,
        op: &Op,
        operand_ty: &Types,
        value: BasicValueEnum<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if *op == Op::Deref {
            let pointee = match operand_ty {
                Types::Pointer(inner) => (**inner).clone(),
                other => {
                    return Err(CodegenError::TypeMismatch {
                        expected: "pointer".to_string(),
                        actual: other.to_string(),
                    });
                }
            };
            let llvm_type = self
                .type_converter()
                .to_llvm_type(&pointee, self.module())?;
            return self.load(value.into_pointer_value(), llvm_type, "deref");
        }

        let result: BasicValueEnum = match (operand_ty, op) {
            (Types::Int, Op::Minus | Op::Sub) => {
                let zero = self.context().i64_type().const_zero();
                self.builder()
                    .build_int_sub(zero, value.into_int_value(), "neg")
                    .map_err(CodegenError::from)?
                    .into()
            }
            (Types::Float, Op::Minus | Op::Sub) => self
                .builder()
                .build_float_neg(value.into_float_value(), "fneg")
                .map_err(CodegenError::from)?
                .into(),
            (Types::Bool, Op::Not) => {
                let ones = self.context().bool_type().const_all_ones();
                self.builder()
                    .build_xor(value.into_int_value(), ones, "not")
                    .map_err(CodegenError::from)?
                    .into()
            }
            (Types::Int, Op::BitNot) => {
                let ones = value.into_int_value().get_type().const_all_ones();
                self.builder()
                    .build_xor(value.into_int_value(), ones, "bitnot")
                    .map_err(CodegenError::from)?
                    .into()
            }
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Unsupported unary operation: {op:?} {operand_ty:?}"),
                });
            }
        };
        Ok(result)
    }

    pub fn build_cast(
        &mut self,
        value: BasicValueEnum<'ctx>,
        source_ty: &Types,
        target_ty: &Types,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let target = self
            .type_converter()
            .to_llvm_type(target_ty, self.module())?;
        let casted: BasicValueEnum = match (source_ty, target_ty) {
            (Types::Int, Types::Float) => self
                .builder()
                .build_signed_int_to_float(value.into_int_value(), target.into_float_type(), "cast")
                .map_err(CodegenError::from)?
                .into(),
            (Types::Char, Types::Int) => self
                .builder()
                .build_int_cast(value.into_int_value(), target.into_int_type(), "cast")
                .map_err(CodegenError::from)?
                .into(),
            (Types::Char, Types::Float) => self
                .builder()
                .build_unsigned_int_to_float(
                    value.into_int_value(),
                    target.into_float_type(),
                    "cast",
                )
                .map_err(CodegenError::from)?
                .into(),
            (Types::Float, Types::Int) => self
                .builder()
                .build_float_to_signed_int(value.into_float_value(), target.into_int_type(), "cast")
                .map_err(CodegenError::from)?
                .into(),
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!("Invalid cast from {source_ty:?} to {target_ty:?}"),
                });
            }
        };
        Ok(casted)
    }

    pub fn const_string_ptr(&mut self, value: &str) -> CodegenResult<PointerValue<'ctx>> {
        let const_str = self.context().const_string(value.as_bytes(), true);
        let global = self.module().add_global(const_str.get_type(), None, "str");
        global.set_initializer(&const_str);
        global.set_constant(true);
        global.set_linkage(Linkage::Private);

        let zero = self.context().i32_type().const_zero();
        unsafe {
            self.builder().build_in_bounds_gep(
                const_str.get_type().get_element_type(),
                global.as_pointer_value(),
                &[zero, zero],
                "str_ptr",
            )
        }
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create string pointer".to_string(),
        })
    }

    pub fn struct_field_ptr(
        &mut self,
        base: PointerValue<'ctx>,
        struct_ty: &Types,
        field_index: usize,
    ) -> CodegenResult<PointerValue<'ctx>> {
        let llvm_type = self
            .type_converter()
            .to_llvm_type(struct_ty, self.module())?;
        let zero = self.context().i32_type().const_zero();
        let index = self
            .context()
            .i32_type()
            .const_int(field_index as u64, false);
        unsafe {
            self.builder()
                .build_in_bounds_gep(llvm_type, base, &[zero, index], "field_ptr")
        }
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create struct field pointer".to_string(),
        })
    }

    pub fn array_elem_ptr(
        &mut self,
        base: PointerValue<'ctx>,
        array_ty: &Types,
        index: IntValue<'ctx>,
    ) -> CodegenResult<PointerValue<'ctx>> {
        let llvm_type = self
            .type_converter()
            .to_llvm_type(array_ty, self.module())?;
        let zero = self.context().i32_type().const_zero();
        unsafe {
            self.builder()
                .build_in_bounds_gep(llvm_type, base, &[zero, index], "elem_ptr")
        }
        .map_err(|_| CodegenError::LLVMBuild {
            message: "Failed to create array element pointer".to_string(),
        })
    }
}
