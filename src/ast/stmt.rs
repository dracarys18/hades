use crate::{
    ast::{Program, Types},
    error::Span,
    impl_span,
    tokens::Ident,
};
use derive_more::Debug;
use indexmap::IndexMap;

use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Program,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Program, span: Span) -> Self {
        Self { stmts, span }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Let {
    pub name: Ident,
    pub declared_type: Option<Types>,
    pub value: ExprAst,
    #[debug(skip)]
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Continue {
    #[debug(skip)]
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ExprAst {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct If {
    pub cond: ExprAst,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct While {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct For {
    pub init: ExprAst,
    pub cond: ExprAst,
    pub update: ExprAst,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDef {
    pub name: Ident,
    pub fields: IndexMap<Ident, Types>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FuncDef {
    pub name: Ident,
    pub params: Vec<(Ident, Types)>,
    pub return_type: Types,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Return {
    pub expr: Option<ExprAst>,
    pub span: Span,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Stmt {
    Let(Let),
    Continue(Continue),
    Expr(ExprAst),
    If(If),
    While(While),
    For(For),
    StructDef(StructDef),
    FuncDef(FuncDef),
    Block(Block),
    Return(Return),
}

impl_span!(Let);
impl_span!(Continue);
impl_span!(ExprAst);
impl_span!(If);
impl_span!(While);
impl_span!(For);
impl_span!(StructDef);
impl_span!(FuncDef);
impl_span!(Return);
impl_span!(Block);

impl Stmt {
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Let(le) => le.span(),
            Stmt::Continue(cont) => cont.span(),
            Stmt::Expr(expr) => expr.span(),
            Stmt::If(i) => i.span(),
            Stmt::While(w) => w.span(),
            Stmt::For(f) => f.span(),
            Stmt::StructDef(s) => s.span(),
            Stmt::FuncDef(f) => f.span(),
            Stmt::Block(b) => b.span(),
            Stmt::Return(r) => r.span(),
        }
    }
}
