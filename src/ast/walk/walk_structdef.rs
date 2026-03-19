use crate::ast::{FieldKind, StructDef, Types, WalkAst};
use crate::error::{SemanticError, Span};
use crate::tokens::Ident;
use crate::typed_ast::{FunctionSignature, TypedFieldKind, TypedFuncDef, TypedStructDef};
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

        // --- Pass 1: register var-only skeleton so that method bodies can resolve self's fields ---
        let var_only: IndexMap<Ident, TypedFieldKind> = self
            .fields
            .iter()
            .filter_map(|(k, v)| match v {
                FieldKind::Var(t) => Some((k.clone(), TypedFieldKind::Var(t.clone()))),
                FieldKind::Func(_) => None,
            })
            .collect();
        ctx.insert_struct(name.clone(), var_only);

        // --- Pass 2: walk all fields (var fields trivially, func fields fully) ---
        let mut fields: IndexMap<Ident, TypedFieldKind> = IndexMap::new();
        for (k, v) in &self.fields {
            match v {
                FieldKind::Var(t) => {
                    fields.insert(k.clone(), TypedFieldKind::Var(t.clone()));
                }
                FieldKind::Func(func_def) => {
                    // Rename to mangled name: StructName__MethodName
                    let mangled = mangle_method_name(&name, &func_def.name, span.clone());
                    let mut mangled_func = func_def.clone();
                    mangled_func.name = mangled;

                    let typed = mangled_func.walk(ctx, span.clone())?;
                    // Store in the struct fields map under the original (unmangled) method name
                    // so that MethodCall can look it up by original name.
                    fields.insert(k.clone(), TypedFieldKind::Func(typed));
                }
            }
        }

        // --- Update struct registration with complete fields (still only vars matter for layout) ---
        // (The var-only skeleton is sufficient; re-inserting has no effect on existing var entries.)

        Ok(TypedStructDef {
            name: name.clone(),
            fields,
            span: self.span.clone(),
        })
    }
}

/// Returns the mangled LLVM function name for a struct method: `StructName__MethodName`.
pub fn mangle_method_name(struct_name: &Ident, method_name: &Ident, span: Span) -> Ident {
    let mangled = format!("{}__{}", struct_name.inner(), method_name.inner());
    Ident::new(mangled, span)
}
