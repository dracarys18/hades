use hades_ast::{TypedStmt, Types};
use hades_tokens::Op;

use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};
use crate::mir::builder::MirBuilder;
use crate::mir::place::{Operand, Place};

pub enum Rvalue {
    Use(Operand),
    BinaryOp(Op, Operand, Operand),
    UnaryOp(Op, Operand),
    Cast(Operand, Types),
}

pub enum MirStmt {
    Assign(Place, Rvalue),
}

impl ToMir for TypedStmt {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        match self {
            TypedStmt::Let(l) => l.to_mir(builder, block),
            TypedStmt::Return(r) => r.to_mir(builder, block),
            TypedStmt::TypedExpr(e) => {
                let mut block = block;
                unpack!(block = e.expr.to_mir(builder, block));
                block.unit()
            }
            _ => block.unit(),
        }
    }
}
