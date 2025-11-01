use crate::ast::{ArrayType, Types};
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::tokens::Ident;
use crate::typed_ast::CompilerContext;
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::{
    AnyTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, StructType,
};

pub struct TypeConverter<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeConverter<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub fn to_llvm_type(
        &mut self,
        ty: &Types,
        compiler_ctx: &CompilerContext,
    ) -> CodegenResult<BasicTypeEnum<'ctx>> {
        let llvm_type = match ty {
            Types::Int => self.context.i64_type().into(),
            Types::Float => self.context.f64_type().into(),
            Types::Bool => self.context.bool_type().into(),
            Types::String => self.context.ptr_type(AddressSpace::default()).into(),
            Types::Struct(name) => self.convert_struct_type(name, compiler_ctx)?.into(),
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
            },
        };

        Ok(llvm_type)
    }

    pub fn convert_struct_type(
        &mut self,
        name: &Ident,
        compiler_ctx: &CompilerContext,
    ) -> CodegenResult<StructType<'ctx>> {
        let struct_def =
            compiler_ctx
                .get_struct_type(&name)
                .map_err(|_| CodegenError::TypeConversion {
                    from: format!("struct {name}"),
                    to: "LLVM type".to_string(),
                })?;

        let mut field_types = Vec::new();
        for (_, field_type) in struct_def.iter() {
            let llvm_field_type = self.to_llvm_type(field_type, compiler_ctx)?;
            field_types.push(llvm_field_type);
        }

        let struct_type = self.context.struct_type(&field_types, false);
        Ok(struct_type)
    }

    pub fn fn_type(&mut self, typ: &AnyTypeEnum<'ctx>) -> FunctionType<'ctx> {
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
        match bits {
            1 => self.context.bool_type(),
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            128 => self.context.i128_type(),
            _ => self.context.custom_width_int_type(bits),
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
        self.context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default())
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
        match (left, right) {
            (Types::Int, Types::Int) | (Types::Float, Types::Float) => true,
            (Types::Bool, Types::Bool) => true,
            (Types::String, Types::String) => true,
            (Types::Int, Types::Float) | (Types::Float, Types::Int) => true,
            _ => false,
        }
    }

    pub fn get_promotion_type(&self, left: &Types, right: &Types) -> Types {
        match (left, right) {
            (Types::Int, Types::Float) | (Types::Float, Types::Int) => Types::Float,
            (t1, t2) if t1 == t2 => t1.clone(),
            _ => left.clone(),
        }
    }
}
