use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::{Ident, Name};
use crate::typed_ast::TypedExpr;
use indexmap::IndexMap;
use inkwell::values::{BasicValueEnum, PointerValue, StructValue};

pub struct StructInit<'a> {
    pub name: &'a Name,
    pub fields: &'a IndexMap<Ident, TypedExpr>,
    pub is_const: bool,
}

impl<'a> StructInit<'a> {
    pub fn new(name: &'a Name, fields: &'a IndexMap<Ident, TypedExpr>, is_const: bool) -> Self {
        Self {
            name,
            fields,
            is_const,
        }
    }
}

impl<'a> Visit for StructInit<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let struct_type = context
            .module()
            .get_struct_type(&self.name.to_string())
            .expect("Struct type should be defined at this point");

        if self.is_const {
            let struct_val = build_const_struct_value(context, struct_type, self.fields)?;
            return Ok(CodegenValue::new(
                struct_val.into(),
                Types::Struct(self.name.clone()),
            ));
        }

        let mut values: Vec<(BasicValueEnum, Types)> = Vec::new();
        for (_field_name, field_expr) in self.fields.iter() {
            let field_val = field_expr.visit(context)?;
            values.push((field_val.value()?, field_expr.get_type()));
        }
        let ptr = build_alloca_struct(context, struct_type, &values)?;
        let struct_val = context.load(ptr, struct_type.into(), "struct_val")?;

        Ok(CodegenValue::new(
            struct_val,
            Types::Struct(self.name.clone()),
        ))
    }
}

fn build_const_struct_value<'ctx>(
    context: &mut LLVMContext<'ctx>,
    struct_type: inkwell::types::StructType<'ctx>,
    fields: &IndexMap<Ident, TypedExpr>,
) -> CodegenResult<StructValue<'ctx>> {
    let mut resolved: Vec<BasicValueEnum<'ctx>> = Vec::new();
    for (_field_name, field_expr) in fields.iter() {
        let val = field_expr.visit(context)?.value()?;
        resolved.push(val);
    }
    Ok(struct_type.const_named_struct(&resolved))
}

fn build_alloca_struct<'ctx>(
    context: &mut LLVMContext<'ctx>,
    struct_type: inkwell::types::StructType<'ctx>,
    values: &[(BasicValueEnum<'ctx>, Types)],
) -> CodegenResult<PointerValue<'ctx>> {
    let struct_ptr = context.create_alloca("struct_alloca", struct_type.into())?;
    let i32_type = context.context().i32_type();
    let zero = i32_type.const_zero();
    for (i, (field_val, field_ast_type)) in values.iter().enumerate() {
        let idx = i32_type.const_int(i as u64, false);
        let field_ptr = unsafe {
            context
                .builder()
                .build_in_bounds_gep(struct_type, struct_ptr, &[zero, idx], "field_ptr")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to GEP struct field: {e}"),
                })?
        };
        context.create_store(field_ptr, *field_val, field_ast_type)?;
    }
    Ok(struct_ptr)
}
