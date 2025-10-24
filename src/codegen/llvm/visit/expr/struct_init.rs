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

        // Allocate space for struct
        let struct_ptr = context
            .builder()
            .build_alloca(struct_type, "struct_tmp")
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to alloca struct: {e}"),
            })?;

        // Iterate over fields
        for (i, (_, field_expr)) in self.fields.iter().enumerate() {
            let field_val = field_expr.visit(context)?;
            let llvm_val = field_val.value;

            let val_to_store = match field_val.type_info {
                // Primitive types — load if pointer
                Types::Int | Types::Float | Types::Bool => {
                    if llvm_val.is_pointer_value() {
                        let ptr_val = llvm_val.into_pointer_value();
                        context
                            .builder()
                            .build_load(ptr_val.get_type(), ptr_val, "load_val")
                            .map_err(|e| CodegenError::LLVMBuild {
                                message: format!("Failed to load primitive: {e}"),
                            })?
                    } else {
                        llvm_val
                    }
                }
                // Strings — keep as pointer
                Types::String => llvm_val,
                // Structs — store by value
                Types::Struct(_) => llvm_val,
                _ => llvm_val,
            };

            let field_ptr = context
                .builder()
                .build_struct_gep(struct_type, struct_ptr, i as u32, "field_ptr")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to get struct field ptr: {e}"),
                })?;

            context
                .builder()
                .build_store(field_ptr, val_to_store)
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
