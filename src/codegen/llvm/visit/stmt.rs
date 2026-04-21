use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedStmt;

impl Visit for TypedStmt {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Self::Let(let_stmt) => let_stmt.visit(context),
            Self::TypedExpr(expr) => expr.visit(context),
            Self::If(if_stmt) => if_stmt.visit(context),
            Self::While(while_stmt) => while_stmt.visit(context),
            Self::For(for_stmt) => for_stmt.visit(context),
            Self::Block(block) => block.visit(context),
            Self::Return(return_stmt) => return_stmt.visit(context),
            Self::Continue(continue_stmt) => continue_stmt.visit(context),
            Self::FuncDef(func_def) => {
                func_def.visit(context)?;
                Ok(())
            }
            Self::StructDef(struct_def) => struct_def.visit(context),
            Self::Break(break_stmt) => break_stmt.visit(context),
            Self::ModuleDecl(_) => Ok(()),
            Self::Import(_) => Ok(()),
            Self::Defer(d) => d.visit(context),
        }
    }
}
