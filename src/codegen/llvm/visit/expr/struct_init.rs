use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::Ident;
use crate::typed_ast::TypedExpr;
use indexmap::IndexMap;

pub struct StructInit<'a> {
    pub name: &'a Ident,
    pub fields: &'a IndexMap<Ident, TypedExpr>,
}

impl<'a> StructInit<'a> {
    pub fn new(name: &'a Ident, fields: &'a IndexMap<Ident, TypedExpr>) -> Self {
        Self { name, fields }
    }
}

impl<'a> Visit for StructInit<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let symbols = context.symbols();
        let struct_type = context
            .type_converter()
            .convert_struct_type(self.name, symbols)?;

        let struct_ptr = context
            .builder()
            .build_alloca(struct_type, "struct_tmp")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to alloca struct: {e}"),
            })?;

        for (i, (_, field_expr)) in self.fields.iter().enumerate() {
            let field_val = field_expr.visit(context)?;
            let field_ptr = context
                .builder()
                .build_struct_gep(struct_type, struct_ptr, i as u32, "field_ptr")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to get field ptr: {e}"),
                })?;

            context
                .builder()
                .build_store(field_ptr, field_val.value)
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to store field: {e}"),
                })?;
        }

        let struct_value = context
            .builder()
            .build_load(struct_type, struct_ptr, "struct_val")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to load struct: {e}"),
            })?;

        Ok(CodegenValue::new(
            struct_value,
            Types::Struct(self.name.clone()),
        ))
    }
}
