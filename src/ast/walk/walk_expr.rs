use crate::ast::{
    ArrayIndexExpr, ArrayType, AssignExpr, AssignTarget, BinaryExpr, Expr, FieldAccessExpr, Types,
    WalkAst,
};
use crate::error::{SemanticError, Span};
use crate::tokens::FunctionName;
use crate::typed_ast::{
    CompilerContext, TypedArrayIndex, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedExprAst, TypedFieldAccess,
};

impl WalkAst for Expr {
    type Output = TypedExpr;

    fn walk(&self, ctx: &mut CompilerContext, span: Span) -> Result<Self::Output, SemanticError> {
        match self {
            Expr::Value(value) => Ok(TypedExpr::Value(value.walk(ctx, span)?)),
            Expr::Ident(ident) => ctx
                .get_variable_type(ident, span)
                .map(|typ| TypedExpr::Ident {
                    ident: ident.clone(),
                    typ,
                }),
            Expr::StructInit { name, fields } => {
                let struct_type = ctx.get_struct_type(name, span.clone())?;
                fields
                    .iter()
                    .map(|(field_name, field_expr)| {
                        let expected = struct_type.get(field_name).ok_or_else(|| {
                            SemanticError::unknown_field(
                                name.clone(),
                                field_name.clone(),
                                span.clone(),
                            )
                        })?;
                        let typed = field_expr.walk(ctx, span.clone())?;
                        (typed.get_type() == expected.get_type())
                            .then(|| (field_name.clone(), typed.clone()))
                            .ok_or_else(|| {
                                SemanticError::type_mismatch(
                                    expected.get_type().to_string(),
                                    typed.get_type().to_string(),
                                    span.clone(),
                                )
                            })
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
            Expr::Call {
                func,
                args,
                receiver,
            } => match receiver {
                Some(recv) => walk_method_call(recv, func, args, ctx, span),
                None => walk_function_call(func, args, ctx, span),
            },
            Expr::FieldAccess(field) => field.walk(ctx, span).map(TypedExpr::FieldAccess),
            Expr::ArrayIndex(index) => index.walk(ctx, span).map(TypedExpr::ArrayIndex),
        }
    }
}

fn walk_typed_args(
    params: &crate::typed_ast::Params,
    args: &[Expr],
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<Vec<TypedExpr>, SemanticError> {
    args.iter()
        .enumerate()
        .map(|(i, arg)| {
            arg.walk(ctx, span.clone()).and_then(|typed| {
                params
                    .type_match(i, &typed.get_type())
                    .then(|| typed.clone())
                    .ok_or_else(|| {
                        SemanticError::type_mismatch(
                            typed.get_type().to_string(),
                            typed.get_type().to_string(),
                            span.clone(),
                        )
                    })
            })
        })
        .collect()
}

fn walk_function_call(
    func: &FunctionName,
    args: &[Expr],
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<TypedExpr, SemanticError> {
    let sig = ctx.get_function_signature(func)?;
    let return_type = sig.return_type().clone();
    let params = sig.params();
    sig.check_arg_count(args.len()).then(|| ()).ok_or_else(|| {
        SemanticError::argument_count_mismatch(
            sig.param_count(),
            args.len(),
            func.to_ident(),
            span.clone(),
        )
    })?;
    walk_typed_args(&params, args, ctx, span).map(|typed_args| TypedExpr::Call {
        func: func.clone(),
        args: typed_args,
        receiver: None,
        typ: return_type,
    })
}

fn walk_method_call(
    receiver: &Expr,
    method: &FunctionName,
    args: &[Expr],
    ctx: &mut CompilerContext,
    span: Span,
) -> Result<TypedExpr, SemanticError> {
    let typed_receiver = receiver.walk(ctx, span.clone())?;
    let mangled = method.mangle(typed_receiver.get_type().unwrap_struct_name());
    let sig = ctx.get_function_signature(&mangled)?;
    let return_type = sig.return_type().clone();
    let params = sig.params();
    sig.check_arg_count(args.len()).then(|| ()).ok_or_else(|| {
        SemanticError::argument_count_mismatch(
            sig.param_count(),
            args.len(),
            mangled.to_ident(),
            span.clone(),
        )
    })?;
    walk_typed_args(&params, args, ctx, span).map(|typed_args| TypedExpr::Call {
        func: mangled,
        args: typed_args,
        receiver: Some(Box::new(typed_receiver)),
        typ: return_type,
    })
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
                let typed_value = self.value.walk(ctx, span.clone())?;
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
                let value = self.value.walk(ctx, span.clone())?;
                ctx.infer_binary_type(&field.field_type, &self.op, &value.get_type(), span)
                    .map(|typ| TypedAssignExpr {
                        target: TypedAssignTarget::FieldAccess(field.clone()),
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
        let right = self.right.walk(ctx, span.clone())?;
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

        match &strc {
            Types::Struct(struct_name) | Types::Array(ArrayType::StructArray(_, struct_name)) => {
                ctx.get_struct_type(struct_name, span.clone())
                    .and_then(|field_map| {
                        field_map
                            .get(&self.field)
                            .ok_or_else(|| {
                                SemanticError::unknown_field(
                                    struct_name.clone(),
                                    self.field.clone(),
                                    span.clone(),
                                )
                            })
                            .map(|field_type| TypedFieldAccess {
                                expr: Box::new(typed_expr.clone()),
                                field: self.field.clone(),
                                struct_type: strc.clone(),
                                field_type: field_type.get_type().clone(),
                            })
                    })
            }
            _ => Err(SemanticError::type_mismatch(
                "Struct".to_string(),
                strc.to_string(),
                span,
            )),
        }
    }
}
