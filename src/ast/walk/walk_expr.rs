use crate::error::SemanticError;
use crate::{
    ast::{Expr, Types, WalkAst},
    typed_ast::{CompilerContext, Params, TypedExpr, TypedExprAst},
};

impl WalkAst for Expr {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        match self {
            Expr::Value(value) => Ok(TypedExpr::Value(value.walk(ctx)?)),
            Expr::Ident(ident) => {
                let typ = ctx.get_variable_type(ident)?;
                Ok(TypedExpr::Ident {
                    ident: ident.clone(),
                    typ,
                })
            }
            Expr::StructInit { name, fields } => {
                let struct_type = ctx.get_struct_type(name)?;
                let mut typed_fields = indexmap::IndexMap::new();

                for (field_name, field_expr) in fields {
                    let expected_type =
                        struct_type
                            .get(field_name)
                            .ok_or_else(|| SemanticError::UnknownField {
                                struct_name: name.clone(),
                                field_name: field_name.clone(),
                            })?;

                    let typed_expr = field_expr.walk(ctx)?;
                    if typed_expr.get_type() != *expected_type {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone().to_string(),
                            found: typed_expr.get_type().to_string(),
                            span: crate::error::Span::default(),
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
            Expr::Binary { left, op, right } => {
                let typed_left = left.walk(ctx)?;
                let typed_right = right.walk(ctx)?;

                let result_type =
                    ctx.infer_binary_type(&typed_left.get_type(), op, &typed_right.get_type())?;

                Ok(TypedExpr::Binary {
                    left: Box::new(typed_left),
                    op: op.clone(),
                    right: Box::new(typed_right),
                    typ: result_type,
                })
            }
            Expr::Unary { op, expr } => {
                let typed_expr = expr.walk(ctx)?;
                let result_type = ctx.infer_unary_type(op, &typed_expr.get_type())?;

                Ok(TypedExpr::Unary {
                    op: op.clone(),
                    expr: Box::new(typed_expr),
                    typ: result_type,
                })
            }
            Expr::Assign { name, op, value } => {
                let var_type = ctx.get_variable_type(name)?;
                let typed_value = value.walk(ctx)?;

                let result_type = ctx.infer_binary_type(&var_type, op, &typed_value.get_type())?;
                Ok(TypedExpr::Assign {
                    name: name.clone(),
                    op: op.clone(),
                    value: Box::new(typed_value),
                    typ: result_type,
                })
            }
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
                    let typed_arg = arg.walk(ctx)?;
                    let expected_type = typed_arg.get_type();

                    let cond = params.type_match(i, &expected_type);
                    if !cond {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone().to_string(),
                            found: typed_arg.get_type().to_string(),
                            span: crate::error::Span::default(),
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
        }
    }
}

impl WalkAst for crate::ast::ExprAst {
    type Output = TypedExprAst;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, crate::error::SemanticError> {
        let typed_expr = self.expr.walk(ctx)?;

        Ok(TypedExprAst {
            expr: typed_expr,
            span: self.span.clone(),
        })
    }
}
