use crate::{ast::Types, error::Span, tokens::Ident};
use derive_more::Debug;
use indexmap::IndexMap;

use super::{TypedProgram, expr::TypedExpr};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBlock {
    pub stmts: TypedProgram,
    #[debug(skip)]
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedLet {
    pub name: Ident,
    pub typ: Types,
    pub value: TypedExprAst,
    #[debug(skip)]
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedContinue {
    #[debug(skip)]
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedExprAst {
    pub expr: TypedExpr,
    pub span: Span,
}

impl TypedExprAst {
    pub fn get_type(&self) -> Types {
        self.expr.get_type()
    }

    pub fn expr(&self) -> &TypedExpr {
        &self.expr
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedIf {
    pub cond: TypedExprAst,
    pub then_branch: TypedBlock,
    pub else_branch: Option<TypedBlock>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedWhile {
    pub cond: TypedExpr,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedFor {
    pub init: TypedExprAst,
    pub cond: TypedExprAst,
    pub update: TypedExprAst,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedStructDef {
    pub name: Ident,
    pub fields: IndexMap<Ident, Types>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedFuncDef {
    pub name: Ident,
    pub params: Vec<(Ident, Types)>,
    pub return_type: Types,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypedReturn {
    pub expr: Option<TypedExprAst>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStmt {
    Let(TypedLet),
    Continue(TypedContinue),
    TypedExpr(TypedExprAst),
    If(TypedIf),
    While(TypedWhile),
    For(TypedFor),
    StructDef(TypedStructDef),
    FuncDef(TypedFuncDef),
    Block(TypedBlock),
    Return(TypedReturn),
}
