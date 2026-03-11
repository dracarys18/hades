use crate::ast::{FieldKind, StructDef, WalkAst};
use crate::error::SemanticError;
use crate::tokens::Ident;
use crate::typed_ast::{TypedFieldKind, TypedFuncDef, TypedStructDef};
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
        let fields = self
            .fields
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.walk(ctx, span.clone())?)))
            .collect::<Result<IndexMap<_, _>, SemanticError>>()?;

        ctx.insert_struct(name.clone(), fields.clone());

        Ok(TypedStructDef {
            name: name.clone(),
            fields: fields.clone(),
            span: self.span.clone(),
        })
    }
}
