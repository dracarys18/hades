use crate::ast::{
    ArrayIndexExpr, ArrayType, AssignExpr, AssignTarget, BinaryExpr, FieldAccessExpr,
};
use crate::error::SemanticError;
use crate::typed_ast::TypedArrayIndex;
use crate::{
    ast::{Expr, Types, WalkAst},
    typed_ast::{
        CompilerContext, Params, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr, TypedExpr,
        TypedExprAst, TypedFieldAccess,
    },
};

impl WalkAst for Expr {
    type Output = TypedExpr;

    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            Expr::Value(value) => Ok(TypedExpr::Value(value.walk(ctx, span)?)),
            Expr::Ident(ident) => {
                let typ = ctx.get_variable_type(ident, span)?;
                Ok(TypedExpr::Ident {
                    ident: ident.clone(),
                    typ,
                })
            }
            Expr::StructInit { name, fields } => {
                let struct_type = ctx.get_struct_type(name, span)?;
                let mut typed_fields = indexmap::IndexMap::new();

                for (field_name, field_expr) in fields {
                    let expected_type =
                        struct_type
                            .get(field_name)
                            .ok_or_else(|| SemanticError::UnknownField {
                                struct_name: name.clone(),
                                field_name: field_name.clone(),
                            })?;

                    let typed_expr = field_expr.walk(ctx, span)?;
                    if typed_expr.get_type() != *expected_type {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone().to_string(),
                            found: typed_expr.get_type().to_string(),
                            span,
                        });
                    }
                    typed_fields.insert(field_name.clone(), typed_expr);
                }

                Ok(TypedExpr::StructInit {
                    name: name.clone(),
                    fields: typed_fields,
                    types: Types::Struct(name.clone()),
                })
            }
            Expr::Binary(binary) => Ok(TypedExpr::Binary(binary.walk(ctx, span)?)),
            Expr::Unary { op, expr } => {
                let typed_expr = expr.walk(ctx, span)?;
                let result_type = ctx.infer_unary_type(op, &typed_expr.get_type(), span)?;

                Ok(TypedExpr::Unary {
                    op: op.clone(),
                    expr: Box::new(typed_expr),
                    typ: result_type,
                })
            }
            Expr::Assign(assign) => Ok(TypedExpr::Assign(assign.walk(ctx, span)?)),
            Expr::Call { func, args } => {
                let sig = ctx.get_function_signature(func)?;

                let return_type = sig.return_type().clone();
                let params = sig.params();
                let param_count = sig.param_count();

                match &params {
                    Params::Variadic => {
                        if args.len() > param_count {
                            return Err(SemanticError::ArgumentCountMismatch {
                                expected: param_count,
                                found: args.len(),
                                function: func.clone(),
                            });
                        }
                    }
                    Params::Fixed(_) => {
                        if args.len() != param_count {
                            return Err(SemanticError::ArgumentCountMismatch {
                                expected: param_count,
                                found: args.len(),
                                function: func.clone(),
                            });
                        }
                    }
                }

                let mut typed_args = Vec::new();

                for (i, arg) in args.iter().enumerate() {
                    let typed_arg = arg.walk(ctx, span)?;
                    let expected_type = typed_arg.get_type();

                    let cond = params.type_match(i, &expected_type);
                    if !cond {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone().to_string(),
                            found: typed_arg.get_type().to_string(),
                            span,
                        });
                    }

                    typed_args.push(typed_arg);
                }

                Ok(TypedExpr::Call {
                    func: func.clone(),
                    args: typed_args,
                    typ: return_type,
                })
            }
            Expr::FieldAccess(field) => Ok(TypedExpr::FieldAccess(field.walk(ctx, span)?)),
            Expr::ArrayIndex(index) => index.walk(ctx, span).map(TypedExpr::ArrayIndex),
        }
    }
}

impl WalkAst for crate::ast::ExprAst {
    type Output = TypedExprAst;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_expr = self.expr.walk(ctx, self.span)?;

        Ok(TypedExprAst {
            expr: typed_expr,
            span: self.span,
        })
    }
}

impl WalkAst for AssignExpr {
    type Output = TypedAssignExpr;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        match self.target {
            AssignTarget::Ident(ref ident) => {
                let var_type = ctx.get_variable_type(ident, span)?;
                let typed_value = self.value.walk(ctx, span)?;
                let result_type =
                    ctx.infer_binary_type(&var_type, &self.op, &typed_value.get_type(), span)?;

                Ok(TypedAssignExpr {
                    target: TypedAssignTarget::Ident(ident.clone()),
                    op: self.op.clone(),
                    value: Box::new(typed_value),
                    typ: result_type,
                })
            }
            AssignTarget::FieldAccess(ref field) => {
                let field = field.walk(ctx, span)?;
                let value = self.value.walk(ctx, span)?;
                let result_type =
                    ctx.infer_binary_type(&field.field_type, &self.op, &value.get_type(), span)?;

                Ok(TypedAssignExpr {
                    target: TypedAssignTarget::FieldAccess(field.clone()),
                    op: self.op.clone(),
                    value: Box::new(value),
                    typ: result_type,
                })
            }
        }
    }
}

impl WalkAst for ArrayIndexExpr {
    type Output = TypedArrayIndex;

    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let typed_expr = self.expr.walk(ctx, span)?;
        let expr_type = typed_expr.get_type();
        let index = self.index.walk(ctx, span)?;

        if Types::Int != index.get_type() {
            return Err(SemanticError::TypeMismatch {
                expected: "Int".to_string(),
                found: index.get_type().to_string(),
                span,
            });
        }

        Ok(TypedArrayIndex {
            expr: Box::new(typed_expr),
            index: Box::new(index),
            typ: expr_type,
        })
    }
}

impl WalkAst for BinaryExpr {
    type Output = TypedBinaryExpr;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_left = self.left.walk(ctx, span)?;
        let typed_right = self.right.walk(ctx, span)?;
        let result_type = ctx.infer_binary_type(
            &typed_left.get_type(),
            &self.op,
            &typed_right.get_type(),
            span,
        )?;
        Ok(TypedBinaryExpr {
            left: Box::new(typed_left),
            op: self.op.clone(),
            right: Box::new(typed_right),
            typ: result_type,
        })
    }
}

impl WalkAst for FieldAccessExpr {
    type Output = TypedFieldAccess;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let typed_expr = self.expr.walk(ctx, span)?;
        let strc = typed_expr.get_type();

        match strc {
            Types::Struct(ref struct_name)
            | Types::Array(ArrayType::StructArray(_, ref struct_name)) => {
                let field = ctx.get_struct_type(struct_name, span)?;
                let field_type = field.get(&self.field).ok_or(SemanticError::UnknownField {
                    struct_name: struct_name.clone(),
                    field_name: self.field.clone(),
                })?;

                Ok(TypedFieldAccess {
                    expr: Box::new(typed_expr),
                    field: self.field.clone(),
                    struct_type: strc,
                    field_type: field_type.clone(),
                })
            }
            _ => Err(SemanticError::TypeMismatch {
                expected: "Struct".to_string(),
                found: strc.to_string(),
                span,
            }),
        }
    }
}
