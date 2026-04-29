use hades_ast::TypedBlock;

use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};
use crate::mir::builder::MirBuilder;
use crate::mir::stmt::MirStmt;
use crate::mir::terminator::Terminator;

pub(crate) struct BasicBlockData {
    pub stmts: Vec<MirStmt>,
    pub terminator: Option<Terminator>,
}

impl BasicBlockData {
    pub fn new() -> Self {
        Self { stmts: Vec::new(), terminator: None }
    }

    pub fn push(&mut self, stmt: MirStmt) {
        self.stmts.push(stmt);
    }
}

impl ToMir for TypedBlock {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        let mut block = block;
        for stmt in &self.stmts {
            unpack!(block = stmt.to_mir(builder, block));
        }
        block.unit()
    }
}
