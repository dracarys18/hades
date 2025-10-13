use crate::{ast::Types, error::Span, tokens::Ident};
use indexmap::IndexMap;

use super::{TypedProgram, expr::TypedExpr};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBlock {
    pub stmts: TypedProgram,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStmt {
    Let {
        name: Ident,
        typ: Types,
        value: TypedExpr,
        span: Span,
    },
    Continue {
        span: Span,
    },
    TypedExpr {
        expr: TypedExpr,
        span: Span,
    },
    If {
        cond: TypedExpr,
        then_branch: TypedProgram,
        else_branch: Option<TypedProgram>,
        span: Span,
    },
    While {
        cond: TypedExpr,
        body: TypedBlock,
        span: Span,
    },
    For {
        init: Box<TypedExpr>,
        cond: Box<TypedExpr>,
        update: Box<TypedExpr>,
        body: TypedBlock,
        span: Span,
    },
    StructDef {
        name: Ident,
        fields: IndexMap<Ident, Types>,
        span: Span,
    },
    FuncDef {
        name: Ident,
        params: Vec<(Ident, Types)>,
        return_type: Types,
        body: TypedBlock,
        span: Span,
    },
    Block {
        stmts: TypedProgram,
        span: Span,
    },
    Return {
        expr: Option<TypedExpr>,
        span: Span,
    },
}
