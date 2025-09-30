use crate::{
    ast::{Program, Types},
    tokens::Ident,
};
use indexmap::IndexMap;

use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: Ident,
        declared_type: Option<Types>,
        value: Expr,
    },
    Continue,
    Expr(Expr), // any expression statement, including assignments
    If {
        cond: Expr,
        then_branch: Program,
        else_branch: Option<Program>,
    },
    While {
        cond: Expr,
        body: Program,
    },
    For {
        init: Box<Stmt>, // initializer
        cond: Expr,      // condition
        update: Expr,    // update expression
        body: Program,   // body statements
    },
    StructDef {
        name: Ident,
        fields: IndexMap<Ident, Types>,
    },
    FuncDef {
        name: Ident,
        params: Vec<(Ident, Types)>,
        return_type: Types,
        body: Program,
    },
    Block(Program),
    Return(Option<Expr>),
}
