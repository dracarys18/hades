use std::num::NonZeroU32;

use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::FunctionSignature;
use hades_ast::{ArrayType, Types};
use hades_tokens::{Name, ParamKind};
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{
    AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType,
    StructType,
};

pub struct TypeConverter<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeConverter<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub fn to_llvm_type(
        &self,
        ty: &Types,
        module: &Module<'ctx>,
    ) -> CodegenResult<BasicTypeEnum<'ctx>> {
        let llvm_type = match ty {
            Types::Int => self.context.i64_type().into(),
            Types::Float => self.context.f64_type().into(),
            Types::Bool => self.context.bool_type().into(),
            Types::String => self.context.ptr_type(AddressSpace::default()).into(),
            Types::Char => self.context.i8_type().into(),
            Types::Struct(name) => self.convert_struct_type(name, module)?.into(),
            Types::Void => {
                return Err(CodegenError::TypeConversion {
                    from: "void".to_string(),
                    to: "basic type".to_string(),
                });
            }
            Types::Generic(_) => {
                return Err(CodegenError::TypeConversion {
                    from: format!("{ty:?}"),
                    to: "LLVM type".to_string(),
                });
            }
            Types::Array(arr) => match arr {
                ArrayType::IntArray(size) => {
                    let elem_type = self.context.i64_type();
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::FloatArray(size) => {
                    let elem_type = self.context.f64_type();
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::BoolArray(size) => {
                    let elem_type = self.context.bool_type();
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::CharArray(size) => {
                    let elem_type = self.context.i8_type();
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::StringArray(size) => {
                    let elem_type = self.context.ptr_type(AddressSpace::default());
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::StructArray(size, name) => {
                    let struct_type = self.convert_struct_type(name, module)?;
                    let array_type = struct_type.array_type(*size as u32);
                    array_type.into()
                }
                ArrayType::PointerArray(size, _) => {
                    let elem_type = self.context.ptr_type(AddressSpace::default());
                    let array_type = elem_type.array_type(*size as u32);
                    array_type.into()
                }
            },
            // Self_ must be resolved to actual type before this pass
            Types::Self_ => {
                return Err(CodegenError::TypeConversion {
                    from: "self".to_string(),
                    to: "LLVM type".to_string(),
                });
            }
            Types::Pointer(_) => self.ptr_type().into(),
        };

        Ok(llvm_type)
    }

    pub fn convert_struct_type(
        &self,
        name: &Name,
        module: &Module<'ctx>,
    ) -> CodegenResult<StructType<'ctx>> {
        module
            .get_struct_type(&name.to_string())
            .ok_or_else(|| CodegenError::TypeConversion {
                from: format!("struct {name}"),
                to: "LLVM type".to_string(),
            })
    }

    pub fn fn_type(&self, typ: &AnyTypeEnum<'ctx>) -> FunctionType<'ctx> {
        match typ {
            AnyTypeEnum::FunctionType(ft) => *ft,
            AnyTypeEnum::IntType(it) => it.fn_type(&[], false),
            AnyTypeEnum::FloatType(ft) => ft.fn_type(&[], false),
            AnyTypeEnum::VoidType(ft) => ft.fn_type(&[], false),
            AnyTypeEnum::StructType(st) => st.fn_type(&[], false),
            _ => panic!("Cannot convert type {typ:?} to FunctionType"),
        }
    }

    pub fn get_int_type(&self, bits: u32) -> IntType<'ctx> {
        let nonzero_bits = NonZeroU32::new(bits).expect("Bit width must be greater than 0");
        match bits {
            1 => self.context.bool_type(),
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            128 => self.context.i128_type(),
            _ => self
                .context
                .custom_width_int_type(nonzero_bits)
                .expect("Failed to create custom width integer type"),
        }
    }

    pub fn get_float_type(&self, bits: u32) -> FloatType<'ctx> {
        match bits {
            16 => self.context.f16_type(),
            32 => self.context.f32_type(),
            64 => self.context.f64_type(),
            128 => self.context.f128_type(),
            _ => panic!("Unsupported float width: {}", bits),
        }
    }

    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.context.void_type()
    }

    pub fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    pub fn is_numeric(&self, ty: &Types) -> bool {
        matches!(ty, Types::Int | Types::Float)
    }

    pub fn is_integer(&self, ty: &Types) -> bool {
        matches!(ty, Types::Int)
    }

    pub fn is_float(&self, ty: &Types) -> bool {
        matches!(ty, Types::Float)
    }

    pub fn is_boolean(&self, ty: &Types) -> bool {
        matches!(ty, Types::Bool)
    }

    pub fn are_compatible(&self, left: &Types, right: &Types) -> bool {
        matches!(
            (left, right),
            (Types::Int, Types::Int)
                | (Types::Float, Types::Float)
                | (Types::Bool, Types::Bool)
                | (Types::String, Types::String)
                | (Types::Int, Types::Float)
                | (Types::Float, Types::Int)
        )
    }

    pub fn get_promotion_type(&self, left: &Types, right: &Types) -> Types {
        match (left, right) {
            (Types::Int, Types::Float) | (Types::Float, Types::Int) => Types::Float,
            (t1, t2) if t1 == t2 => t1.clone(),
            _ => left.clone(),
        }
    }

    pub fn params_to_llvm_types(
        &self,
        sig: &FunctionSignature,
        module: &Module<'ctx>,
    ) -> CodegenResult<Vec<BasicMetadataTypeEnum<'ctx>>> {
        let params = sig.to_fixed_params();
        let mut param_types = Vec::new();
        for (param, declared_type) in &params {
            match param {
                ParamKind::Self_(_) => {
                    param_types.push(self.ptr_type().into());
                }
                ParamKind::Ident(_) => {
                    param_types.push(self.to_llvm_type(declared_type, module)?.into());
                }
            }
        }
        Ok(param_types)
    }
}
