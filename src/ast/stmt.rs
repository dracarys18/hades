use std::fmt::Debug;

use crate::{
    ast::{Program, Types},
    error::Span,
    tokens::Ident,
};
use indexmap::IndexMap;

use super::expr::Expr;

#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    pub stmts: Program,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Program, span: Span) -> Self {
        Self { stmts, span }
    }
    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn stmts(&self) -> &Program {
        &self.stmts
    }
}

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: Ident,
        declared_type: Option<Types>,
        value: Expr,
        span: Span,
    },
    Continue {
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    If {
        cond: Expr,
        then_branch: Program,
        else_branch: Option<Program>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        span: Span,
    },
    For {
        init: Expr,
        cond: Expr,
        update: Expr,
        body: Block,
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
        body: Block,
        span: Span,
    },
    Block(Block),
    Return {
        expr: Option<Expr>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Let { span, .. } => span,
            Stmt::Continue { span } => span,
            Stmt::Expr { span, .. } => span,
            Stmt::If { span, .. } => span,
            Stmt::While { span, .. } => span,
            Stmt::For { span, .. } => span,
            Stmt::StructDef { span, .. } => span,
            Stmt::FuncDef { span, .. } => span,
            Stmt::Block(block) => block.span(),
            Stmt::Return { span, .. } => span,
        }
    }
}

impl Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Let {
                name,
                declared_type,
                value,
                ..
            } => f
                .debug_struct("Let")
                .field("name", name)
                .field("declared_type", declared_type)
                .field("value", value)
                .finish(),
            Stmt::Continue { .. } => f.debug_struct("Continue").finish(),
            Stmt::Expr { expr, .. } => f.debug_struct("Expr").field("expr", expr).finish(),
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => f
                .debug_struct("If")
                .field("cond", cond)
                .field("then_branch", then_branch)
                .field("else_branch", else_branch)
                .finish(),
            Stmt::While { cond, body, .. } => f
                .debug_struct("While")
                .field("cond", cond)
                .field("body", body)
                .finish(),
            Stmt::For {
                init,
                cond,
                update,
                body,
                ..
            } => f
                .debug_struct("For")
                .field("init", init)
                .field("cond", cond)
                .field("update", update)
                .field("body", body)
                .finish(),
            Stmt::StructDef { name, fields, .. } => f
                .debug_struct("StructDef")
                .field("name", name)
                .field("fields", fields)
                .finish(),
            Stmt::FuncDef {
                name,
                params,
                return_type,
                body,
                ..
            } => f
                .debug_struct("FuncDef")
                .field("name", name)
                .field("params", params)
                .field("return_type", return_type)
                .field("body", body)
                .finish(),
            Stmt::Block(block) => f
                .debug_struct("Block")
                .field("stmts", block.stmts())
                .finish(),
            Stmt::Return { expr, .. } => f.debug_struct("Return").field("expr", expr).finish(),
        }
    }
}
