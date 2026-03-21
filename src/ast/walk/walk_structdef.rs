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

        // --- Pass 1: register var-only skeleton so method bodies can resolve self's fields ---
        let var_only = self
            .fields
            .iter()
            .filter_map(|(k, v)| match v {
                FieldKind::Var(t) => Some((k.clone(), TypedFieldKind::Var(t.clone()))),
                FieldKind::Func(_) => None,
            })
            .collect();
        ctx.insert_struct(name.clone(), var_only);

        // --- Pass 2: walk all fields ---
        let mut fields = IndexMap::new();
        for (k, v) in &self.fields {
            match v {
                FieldKind::Var(t) => {
                    fields.insert(k.clone(), TypedFieldKind::Var(t.clone()));
                }
                FieldKind::Func(func_def) => {
                    let mangled = func_def.name.mangle(&name);
                    let mut mangled_func = func_def.clone();
                    mangled_func.name = mangled;

                    let typed = mangled_func.walk(ctx, span.clone())?;
                    // Store under the original (unmangled) method name so call-site lookup works.
                    fields.insert(k.clone(), TypedFieldKind::Func(typed));
                }
            }
        }

        Ok(TypedStructDef {
            name: name.clone(),
            fields,
            span: self.span.clone(),
        })
    }
}
