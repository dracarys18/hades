use crate::ast::Types;
use inkwell::builder::BuilderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("Type conversion failed: {from} -> {to}")]
    TypeConversion { from: String, to: String },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    #[error("Function not found: {name}")]
    FunctionNotFound { name: String },

    #[error("LLVM build error: {message}")]
    LLVMBuild { message: String },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Invalid scope operation: {operation}")]
    ScopeError { operation: String },

    #[error("Variable already declared: {name}")]
    VariableRedeclaration { name: String },

    #[error("Cannot pop global scope")]
    GlobalScopePop,

    #[error("Invalid array access: index {index} for array of size {size}")]
    ArrayBounds { index: usize, size: usize },

    #[error("Invalid struct field: {field} in struct {struct_name}")]
    InvalidField { field: String, struct_name: String },
}

impl From<BuilderError> for CodegenError {
    fn from(err: BuilderError) -> Self {
        CodegenError::LLVMBuild {
            message: format!("Builder error: {:?}", err),
        }
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;

#[derive(Debug, Clone)]
pub struct CodegenValue<'ctx> {
    pub value: inkwell::values::BasicValueEnum<'ctx>,
    pub type_info: Types,
}

impl<'ctx> CodegenValue<'ctx> {
    pub fn new(value: inkwell::values::BasicValueEnum<'ctx>, type_info: Types) -> Self {
        Self { value, type_info }
    }

    pub fn int32(context: &'ctx inkwell::context::Context, val: i32) -> Self {
        let llvm_val = context.i32_type().const_int(val as u64, false);
        Self::new(llvm_val.into(), Types::Int)
    }

    pub fn float64(context: &'ctx inkwell::context::Context, val: f64) -> Self {
        let llvm_val = context.f64_type().const_float(val);
        Self::new(llvm_val.into(), Types::Float)
    }

    pub fn bool_val(context: &'ctx inkwell::context::Context, val: bool) -> Self {
        let llvm_val = context.bool_type().const_int(val as u64, false);
        Self::new(llvm_val.into(), Types::Bool)
    }
}
