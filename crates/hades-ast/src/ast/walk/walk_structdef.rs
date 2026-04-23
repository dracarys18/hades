use crate::ast::{FieldKind, StructDef, WalkAst};
use crate::typed_ast::{TypedFieldKind, TypedStructDef};
use hades_error::SemanticError;
use indexmap::IndexMap;

impl WalkAst for FieldKind {
    type Output = TypedFieldKind;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: hades_error::Span,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            FieldKind::Var(typ) => Ok(TypedFieldKind::Var(typ.qualify(ctx.module_name()))),
            FieldKind::Func(func_def) => {
                Ok(TypedFieldKind::Func(Box::new(func_def.walk(ctx, span)?)))
            }
        }
    }
}

impl WalkAst for StructDef {
    type Output = TypedStructDef;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: hades_error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let name = self.name.full_name_optional(ctx.module_name());

        let var_fields = self
            .fields
            .iter()
            .filter_map(|(k, v)| match v {
                FieldKind::Var(t) => {
                    Some((k.clone(), TypedFieldKind::Var(t.qualify(ctx.module_name()))))
                }
                FieldKind::Func(_) => None,
            })
            .collect();
        ctx.insert_struct(name.clone(), var_fields);

        for (_, v) in &self.fields {
            if let FieldKind::Func(func_def) = v {
                func_def.register(ctx)?;
            }
        }

        let fields = self
            .fields
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.walk(ctx, span.clone())?)))
            .collect::<Result<IndexMap<_, _>, _>>()?;

        Ok(TypedStructDef {
            name,
            fields,
            span: self.span.clone(),
        })
    }
}
