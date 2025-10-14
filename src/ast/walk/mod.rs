mod walk_block;
mod walk_continue;
mod walk_expr;
mod walk_for;
mod walk_func;
mod walk_if;
mod walk_let;
mod walk_program;
mod walk_return;
mod walk_stmt;
mod walk_structdef;
mod walk_while;

use crate::ast::Stmt;
use crate::error::SemanticError;
use crate::typed_ast::*;

pub trait WalkAst {
    type Output;
    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError>;
}
