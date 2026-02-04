use crate::ast::{Stmt, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedStmt};

impl WalkAst for Stmt {
    type Output = TypedStmt;

    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        match self {
            Stmt::Let(decl) => decl.walk(ctx).map(TypedStmt::Let),
            Stmt::Continue(cont) => cont.walk(ctx).map(TypedStmt::Continue),
            Stmt::Expr(expr) => expr.walk(ctx).map(TypedStmt::TypedExpr),
            Stmt::If(i) => i.walk(ctx).map(TypedStmt::If),
            Stmt::While(whil) => whil.walk(ctx).map(TypedStmt::While),
            Stmt::For(fo) => fo.walk(ctx).map(TypedStmt::For),
            Stmt::StructDef(st) => st.walk(ctx).map(TypedStmt::StructDef),
            Stmt::FuncDef(f) => f.walk(ctx).map(TypedStmt::FuncDef),
            Stmt::Block(block) => block.walk(ctx).map(TypedStmt::Block),
            Stmt::Return(ret) => ret.walk(ctx).map(TypedStmt::Return),
            Stmt::ModuleDecl(m) => m.walk(ctx).map(TypedStmt::ModuleDecl),
            Stmt::Import(import) => import.walk(ctx).map(TypedStmt::Import),
        }
    }
}
