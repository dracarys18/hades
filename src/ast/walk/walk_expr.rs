use crate::ast::{
    ArrayIndexExpr, ArrayType, AssignExpr, AssignTarget, BinaryExpr, CallKind, Expr,
    FieldAccessExpr, NullExpr, StructInitExpr, Types, WalkAst,
};
use crate::error::{SemanticError, Span};
use crate::tokens::{Ident, Name};
use crate::typed_ast::{
    CompilerContext, TypedArrayIndex, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedExprAst, TypedFieldAccess,
};

use super::walk_possibly_null;

impl WalkAst for Expr {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        match self {
            Expr::Null => NullExpr::new(None).walk(ctx, span),
            Expr::Value(value) => Ok(TypedExpr::Value(value.walk(ctx, span)?)),
            Expr::Ident(ident) => ctx
                .get_variable_type(ident, span)
                .map(|typ| TypedExpr::Ident {
                    ident: ident.clone(),
                    typ,
                }),
            Expr::StructInit(StructInitExpr {
                name,
                fields,
                module,
            }) => {
                let name = name.full_name_optional(
                    module
                        .as_ref()
                        .map(|m| m.inner())
                        .or_else(|| ctx.module_name()),
                );
                let struct_type = ctx.get_struct_type(&name, span.clone())?;

                fields
                    .iter()
                    .map(|(field_name, field_expr)| {
                        let expected = struct_type.get(field_name).ok_or_else(|| {
                            SemanticError::unknown_field(
                                name.to_ident().clone(),
                                field_name.clone(),
                                span.clone(),
                            )
                        })?;
                        let expected_type = expected.get_type();

                        let typed = walk_possibly_null(
                            field_expr,
                            Some(expected_type.clone()),
                            ctx,
                            span.clone(),
                        )?;

                        let typed_type = typed.get_type();
                        if typed_type != expected_type {
                            return Err(SemanticError::type_mismatch(
                                expected_type.to_string(),
                                typed_type.to_string(),
                                span.clone(),
                            ));
                        }
                        Ok((field_name.clone(), typed))
                    })
                    .collect::<Result<_, _>>()
                    .map(|typed_fields| TypedExpr::StructInit {
                        name: name.clone(),
                        fields: typed_fields,
                        types: Types::Struct(name.clone()),
                    })
            }
            Expr::Binary(binary) => binary.walk(ctx, span).map(TypedExpr::Binary),
            Expr::Unary { op, expr } => {
                let typed = expr.walk(ctx, span.clone())?;
                ctx.infer_unary_type(op, &typed.get_type(), span.clone())
                    .map(|typ| TypedExpr::Unary {
                        op: op.clone(),
                        expr: Box::new(typed),
                        typ,
                    })
            }
            Expr::Assign(assign) => assign.walk(ctx, span).map(TypedExpr::Assign),
            Expr::Call(kind) => match kind {
                CallKind::Function(call) => call.walk(ctx, span),
                CallKind::Method(call) => call.walk(ctx, span),
                CallKind::Qualified(call) => call.walk(ctx, span),
            },
            Expr::FieldAccess(field) => field.walk(ctx, span).map(TypedExpr::FieldAccess),
            Expr::ArrayIndex(index) => index.walk(ctx, span).map(TypedExpr::ArrayIndex),
            Expr::As(as_expr) => as_expr.walk(ctx, span).map(TypedExpr::As),
        }
    }
}

impl WalkAst for crate::ast::ExprAst {
    type Output = TypedExprAst;
    fn walk(&self, ctx: &mut CompilerContext, _span: Span) -> Result<Self::Output, SemanticError> {
        self.expr
            .walk(ctx, self.span.clone())
            .map(|expr| TypedExprAst {
                expr,
                span: self.span.clone(),
            })
    }
}

impl WalkAst for AssignExpr {
    type Output = TypedAssignExpr;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        match self.target {
            AssignTarget::Ident(ref ident) => {
                let var_type = ctx.get_variable_type(ident, span.clone())?;
                let typed_value =
                    walk_possibly_null(&self.value, Some(var_type.clone()), ctx, span.clone())?;
                ctx.infer_binary_type(&var_type, &self.op, &typed_value.get_type(), span)
                    .map(|typ| TypedAssignExpr {
                        target: TypedAssignTarget::Ident(ident.clone()),
                        op: self.op.clone(),
                        value: Box::new(typed_value),
                        typ,
                    })
            }
            AssignTarget::FieldAccess(ref field) => {
                let field = field.walk(ctx, span.clone())?;
                let value = walk_possibly_null(
                    &self.value,
                    Some(field.field_type.clone()),
                    ctx,
                    span.clone(),
                )?;
                ctx.infer_binary_type(&field.field_type, &self.op, &value.get_type(), span)
                    .map(|typ| TypedAssignExpr {
                        target: TypedAssignTarget::FieldAccess(field.clone()),
                        op: self.op.clone(),
                        value: Box::new(value),
                        typ,
                    })
            }
            AssignTarget::ArrayIndex(ref index) => {
                let typed_index = index.walk(ctx, span.clone())?;
                let elem_type = typed_index.typ.get_array_elem_type();
                let value = self.value.walk(ctx, span.clone())?;
                ctx.infer_binary_type(&elem_type, &self.op, &value.get_type(), span)
                    .map(|typ| TypedAssignExpr {
                        target: TypedAssignTarget::ArrayIndex(typed_index),
                        op: self.op.clone(),
                        value: Box::new(value),
                        typ,
                    })
            }
            AssignTarget::Deref(ref inner_expr) => {
                let typed_inner = inner_expr.walk(ctx, span.clone())?;
                let pointee_type = typed_inner.get_deref_type(span.clone())?;
                let value =
                    walk_possibly_null(&self.value, Some(pointee_type.clone()), ctx, span.clone())?;
                ctx.infer_binary_type(&pointee_type, &self.op, &value.get_type(), span)
                    .map(|typ| TypedAssignExpr {
                        target: TypedAssignTarget::Deref(Box::new(typed_inner)),
                        op: self.op.clone(),
                        value: Box::new(value),
                        typ,
                    })
            }
        }
    }
}

impl WalkAst for ArrayIndexExpr {
    type Output = TypedArrayIndex;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let typed_expr = self.expr.walk(ctx, span.clone())?;
        let expr_type = typed_expr.get_type();
        let index = self.index.walk(ctx, span.clone())?;
        let index_type = index.get_type();

        (Types::Int == index_type)
            .then(|| TypedArrayIndex {
                expr: Box::new(typed_expr),
                index: Box::new(index),
                typ: expr_type,
            })
            .ok_or_else(|| {
                SemanticError::type_mismatch("Int".to_string(), index_type.to_string(), span)
            })
    }
}

impl WalkAst for BinaryExpr {
    type Output = TypedBinaryExpr;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let left = self.left.walk(ctx, span.clone())?;
        let right = walk_possibly_null(&self.right, Some(left.get_type()), ctx, span.clone())?;
        ctx.infer_binary_type(&left.get_type(), &self.op, &right.get_type(), span)
            .map(|typ| TypedBinaryExpr {
                left: Box::new(left),
                op: self.op.clone(),
                right: Box::new(right),
                typ,
            })
    }
}

impl WalkAst for FieldAccessExpr {
    type Output = TypedFieldAccess;
    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        let typed_expr = self.expr.walk(ctx, span.clone())?;
        let strc = typed_expr.get_type();

        let walk_struct = |struct_name: &Name| {
            ctx.get_struct_type(&struct_name, span.clone())
                .and_then(|field_map| {
                    field_map
                        .get(&self.field)
                        .ok_or_else(|| {
                            SemanticError::unknown_field(
                                struct_name.to_ident().clone(),
                                self.field.clone(),
                                span.clone(),
                            )
                        })
                        .map(|field_type| TypedFieldAccess {
                            expr: Box::new(typed_expr.clone()),
                            field: self.field.clone(),
                            struct_type: Types::Struct(struct_name.clone()),
                            field_type: field_type.get_type().clone(),
                        })
                })
        };
        match &strc {
            Types::Struct(struct_name) | Types::Array(ArrayType::StructArray(_, struct_name)) => {
                walk_struct(struct_name)
            }
            Types::Pointer(inner) => match inner.as_ref() {
                Types::Struct(struct_name)
                | Types::Array(ArrayType::StructArray(_, struct_name)) => walk_struct(struct_name),
                other => Err(SemanticError::type_mismatch(
                    "Struct or pointer to struct".to_string(),
                    other.to_string(),
                    span,
                )),
            },
            _ => Err(SemanticError::type_mismatch(
                "Struct".to_string(),
                strc.to_string(),
                span,
            )),
        }
    }
}
