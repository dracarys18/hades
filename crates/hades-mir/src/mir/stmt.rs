use hades_ast::TypedStmt;
use hades_error::Span;

use crate::mir::builder::MirBuilder;
use crate::mir::place::Place;
use crate::mir::rvalue::Rvalue;
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

#[derive(Debug, Clone)]
pub enum StatementKind {
    Assign(Place, Box<Rvalue>),
    Nop,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

impl Statement {
    pub fn assign(place: Place, rvalue: Rvalue, span: Span) -> Self {
        Self {
            kind: StatementKind::Assign(place, Box::new(rvalue)),
            span,
        }
    }

    pub fn nop(span: Span) -> Self {
        Self {
            kind: StatementKind::Nop,
            span,
        }
    }
}

impl ToMir for TypedStmt {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder<'_>, block: BasicBlock) -> BlockAnd<()> {
        match self {
            TypedStmt::Let(l) => l.to_mir(builder, block),
            TypedStmt::Return(r) => r.to_mir(builder, block),
            TypedStmt::TypedExpr(e) => {
                let mut block = block;
                unpack!(block = e.expr.to_mir(builder, block));
                block.unit()
            }
            TypedStmt::If(s) => s.to_mir(builder, block),
            TypedStmt::While(s) => s.to_mir(builder, block),
            TypedStmt::For(s) => s.to_mir(builder, block),
            TypedStmt::Block(b) => b.to_mir(builder, block),
            TypedStmt::Continue(s) => s.to_mir(builder, block),
            TypedStmt::Break(s) => s.to_mir(builder, block),
            TypedStmt::Defer(s) => s.to_mir(builder, block),
            TypedStmt::FuncDef(_)
            | TypedStmt::StructDef(_)
            | TypedStmt::ModuleDecl(_)
            | TypedStmt::Import(_) => block.unit(),
        }
    }
}
