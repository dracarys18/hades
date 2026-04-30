use hades_ast::{TypedStmt, Types};
use hades_tokens::{Name, Op};

use crate::mir::builder::MirBuilder;
use crate::mir::place::{Operand, Place};
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

pub enum Rvalue {
    Use(Operand),
    BinaryOp(Op, Operand, Operand),
    UnaryOp(Op, Operand),
    Cast(Operand, Types),
    Aggregate(Name, Vec<Operand>),
}

pub enum MirStmt {
    Assign(Place, Rvalue),
}

impl ToMir for TypedStmt {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        match self {
            TypedStmt::Let(l) => l.to_mir(builder, block),
            TypedStmt::Return(r) => r.to_mir(builder, block),
            TypedStmt::TypedExpr(e) => {
                unpack!(block = e.expr.to_mir(builder, block));
                block.unit()
            }
            _ => unimplemented!(),
        }
    }
}
