use crate::ast::{Stmt, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedStmt};

impl WalkAst for Stmt {
    type Output = TypedStmt;

    fn walk(
        &self,
        ctx: &mut CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            Stmt::Let(decl) => decl.walk(ctx, span).map(TypedStmt::Let),
            Stmt::Continue(cont) => cont.walk(ctx, span).map(TypedStmt::Continue),
            Stmt::Expr(expr) => expr.walk(ctx, span).map(TypedStmt::TypedExpr),
            Stmt::If(i) => i.walk(ctx, span).map(TypedStmt::If),
            Stmt::While(whil) => whil.walk(ctx, span).map(TypedStmt::While),
            Stmt::For(fo) => fo.walk(ctx, span).map(TypedStmt::For),
            Stmt::StructDef(st) => st.walk(ctx, span).map(TypedStmt::StructDef),
            Stmt::FuncDef(f) => f.walk(ctx, span).map(TypedStmt::FuncDef),
            Stmt::Block(block) => block.walk(ctx, span).map(TypedStmt::Block),
            Stmt::Return(ret) => ret.walk(ctx, span).map(TypedStmt::Return),
            Stmt::ModuleDecl(m) => m.walk(ctx, span).map(TypedStmt::ModuleDecl),
            Stmt::Import(import) => import.walk(ctx, span).map(TypedStmt::Import),
        }
    }
}
