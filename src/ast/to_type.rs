use crate::ast::{Expr, Program, Stmt, Types};
use crate::error::SemanticError;
use crate::typed_ast::{TypeContext, TypedExpr, TypedProgram, TypedStmt};

pub trait ToTyped {
    type Output;
    fn to_typed(&self, ctx: &mut TypeContext) -> Result<Self::Output, SemanticError>;
}

impl ToTyped for Stmt {
    type Output = TypedStmt;

    fn to_typed(&self, ctx: &mut TypeContext) -> Result<Self::Output, SemanticError> {
        match self {
            Stmt::Let {
                name,
                declared_type,
                value,
                span,
            } => {
                let typed_value = value.to_typed(ctx)?;
                let inferred_type = typed_value.get_type();

                let final_type = match declared_type {
                    Some(declared) => {
                        if declared != &inferred_type {
                            return Err(SemanticError::TypeMismatch {
                                expected: declared.to_string(),
                                found: inferred_type.to_string(),
                                span: span.clone(),
                            });
                        }
                        declared.clone()
                    }
                    None => inferred_type,
                };

                ctx.insert_variable(name.clone(), final_type.clone());

                Ok(TypedStmt::Let {
                    name: name.clone(),
                    typ: final_type,
                    value: typed_value,
                    span: span.clone(),
                })
            }
            Stmt::Continue { span } => Ok(TypedStmt::Continue { span: span.clone() }),
            Stmt::Expr { expr, span } => {
                let typed_expr = expr.to_typed(ctx)?;
                Ok(TypedStmt::TypedExpr {
                    expr: typed_expr,
                    span: span.clone(),
                })
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                let typed_cond = cond.to_typed(ctx)?;
                if typed_cond.get_type() != Types::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Types::Bool.to_string(),
                        found: typed_cond.get_type().to_string(),
                        span: span.clone(),
                    });
                }

                let typed_then = then_branch.to_typed(ctx)?;
                let typed_else = match else_branch {
                    Some(else_stmts) => Some(else_stmts.to_typed(ctx)?),
                    None => None,
                };

                Ok(TypedStmt::If {
                    cond: typed_cond,
                    then_branch: typed_then,
                    else_branch: typed_else,
                    span: span.clone(),
                })
            }
            Stmt::While { cond, body, span } => {
                let typed_cond = cond.to_typed(ctx)?;
                if typed_cond.get_type() != Types::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Types::Bool.to_string(),
                        found: typed_cond.get_type().to_string(),
                        span: span.clone(),
                    });
                }

                let typed_body = body.to_typed(ctx)?;

                Ok(TypedStmt::While {
                    cond: typed_cond,
                    body: typed_body,
                    span: span.clone(),
                })
            }
            Stmt::For {
                init,
                cond,
                update,
                body,
                span,
            } => {
                let typed_init = init.to_typed(ctx)?;
                let typed_cond = cond.to_typed(ctx)?;
                let typed_update = update.to_typed(ctx)?;

                if typed_cond.get_type() != Types::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Types::Bool.to_string(),
                        found: typed_cond.get_type().to_string(),
                        span: span.clone(),
                    });
                }

                let typed_body = body.to_typed(ctx)?;

                Ok(TypedStmt::For {
                    init: Box::new(typed_init),
                    cond: Box::new(typed_cond),
                    update: Box::new(typed_update),
                    body: typed_body,
                    span: span.clone(),
                })
            }
            Stmt::StructDef { name, fields, span } => {
                ctx.insert_struct(name.clone(), fields.clone());
                Ok(TypedStmt::StructDef {
                    name: name.clone(),
                    fields: fields.clone(),
                    span: span.clone(),
                })
            }
            Stmt::FuncDef {
                name,
                params,
                return_type,
                body,
                span,
            } => {
                ctx.enter_function(name.clone(), params.clone(), return_type.clone())?;
                for (param_name, param_type) in params {
                    ctx.insert_variable(param_name.clone(), param_type.clone());
                }

                let typed_body = body.to_typed(ctx)?;
                ctx.exit_function();

                Ok(TypedStmt::FuncDef {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: typed_body,
                    span: span.clone(),
                })
            }
            Stmt::Block { stmts, span } => {
                ctx.enter_scope();
                let typed_stmts = stmts.to_typed(ctx)?;
                ctx.exit_scope();

                Ok(TypedStmt::Block {
                    stmts: typed_stmts,
                    span: span.clone(),
                })
            }
            Stmt::Return { expr, span } => {
                let typed_expr = match expr {
                    Some(e) => Some(e.to_typed(ctx)?),
                    None => None,
                };

                let return_type = match &typed_expr {
                    Some(e) => e.get_type(),
                    None => Types::Void,
                };

                ctx.check_return_type(return_type, span.clone())?;

                Ok(TypedStmt::Return {
                    expr: typed_expr,
                    span: span.clone(),
                })
            }
        }
    }
}

impl ToTyped for Expr {
    type Output = TypedExpr;

    fn to_typed(&self, ctx: &mut TypeContext) -> Result<Self::Output, SemanticError> {
        match self {
            Expr::Number(n) => Ok(TypedExpr::Number(*n)),
            Expr::Float(f) => Ok(TypedExpr::Float(*f)),
            Expr::String(s) => Ok(TypedExpr::String(s.clone())),
            Expr::Boolean(b) => Ok(TypedExpr::Boolean(*b)),
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

                match &struct_type {
                    Types::Struct {
                        fields: struct_fields,
                        ..
                    } => {
                        for (field_name, field_expr) in fields {
                            let expected_type = struct_fields.get(field_name).ok_or_else(|| {
                                SemanticError::UnknownField {
                                    struct_name: name.clone(),
                                    field_name: field_name.clone(),
                                }
                            })?;

                            let typed_expr = field_expr.to_typed(ctx)?;
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
                            types: struct_type,
                        })
                    }
                    _ => Err(SemanticError::NotAStruct { name: name.clone() }),
                }
            }
            Expr::Binary { left, op, right } => {
                let typed_left = left.to_typed(ctx)?;
                let typed_right = right.to_typed(ctx)?;

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
                let typed_expr = expr.to_typed(ctx)?;
                let result_type = ctx.infer_unary_type(op, &typed_expr.get_type())?;

                Ok(TypedExpr::Unary {
                    op: op.clone(),
                    expr: Box::new(typed_expr),
                    typ: result_type,
                })
            }
            Expr::Assign { name, op, value } => {
                println!("Assigning to variable: {}", name);
                let var_type = ctx.get_variable_type(name)?;
                let typed_value = value.to_typed(ctx)?;

                let result_type = match op {
                    Some(assign_op) => {
                        ctx.infer_binary_type(&var_type, assign_op, &typed_value.get_type())?
                    }
                    None => {
                        if var_type != typed_value.get_type() {
                            return Err(SemanticError::TypeMismatch {
                                expected: var_type.clone().to_string(),
                                found: typed_value.get_type().to_string(),
                                span: crate::error::Span::default(),
                            });
                        }
                        var_type.clone()
                    }
                };

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
                let params = sig.params().to_vec();
                let param_count = sig.param_count();

                if args.len() != param_count {
                    return Err(SemanticError::ArgumentCountMismatch {
                        expected: param_count,
                        found: args.len(),
                        function: func.clone(),
                    });
                }

                let mut typed_args = Vec::new();
                for (arg, expected_type) in args.iter().zip(params.iter()) {
                    let typed_arg = arg.to_typed(ctx)?;
                    if typed_arg.get_type() != *expected_type {
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

impl ToTyped for Program {
    type Output = TypedProgram;

    fn to_typed(&self, ctx: &mut TypeContext) -> Result<Self::Output, SemanticError> {
        let mut typed_stmts = Vec::new();
        for stmt in self.iter() {
            typed_stmts.push(stmt.to_typed(ctx)?);
        }
        Ok(crate::typed_ast::TypedProgram::new(typed_stmts))
    }
}
