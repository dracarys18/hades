use crate::ast::{FieldKind, StructDef, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{TypedFieldKind, TypedStructDef};
use indexmap::IndexMap;

impl WalkAst for FieldKind {
    type Output = TypedFieldKind;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            FieldKind::Var(typ) => Ok(TypedFieldKind::Var(typ.clone())),
            FieldKind::Func(func_def) => {
                let typed_func_def = func_def.walk(ctx, span.clone())?;
                Ok(TypedFieldKind::Func(typed_func_def))
            }
        }
    }
}

impl WalkAst for StructDef {
    type Output = TypedStructDef;
    fn walk(
        &self,
        ctx: &mut crate::typed_ast::CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let name = self.name.clone();
        let mut fields = IndexMap::new();

        // Pre-register var fields so method bodies can resolve self's field types.
        let var_only = self
            .fields
            .iter()
            .filter_map(|(k, v)| match v {
                FieldKind::Var(t) => Some((k.clone(), TypedFieldKind::Var(t.clone()))),
                FieldKind::Func(_) => None,
            })
            .collect();
        ctx.insert_struct(name.clone(), var_only);

        for (k, v) in &self.fields {
            match v {
                FieldKind::Var(t) => {
                    fields.insert(k.clone(), TypedFieldKind::Var(t.clone()));
                }
                FieldKind::Func(func_def) => {
                    let mut mangled_func = func_def.clone();
                    mangled_func.name = func_def.name.mangle(&name);
                    let typed = mangled_func.walk(ctx, span.clone())?;
                    fields.insert(k.clone(), TypedFieldKind::Func(typed));
                }
            }
        }
        ctx.insert_struct(name.clone(), fields.clone());

        Ok(TypedStructDef {
            name: name.clone(),
            fields,
            span: self.span.clone(),
        })
    }
}
