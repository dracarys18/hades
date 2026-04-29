use hades_ast::TypedLet;

use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};
use crate::mir::builder::MirBuilder;
use crate::mir::place::Place;
use crate::mir::stmt::MirStmt;

impl ToMir for TypedLet {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        let local_idx = builder.build_local(self.name.clone(), self.typ.clone());

        let mut block = block;
        let rvalue = unpack!(block = self.value.expr.to_mir(builder, block));

        builder.push_stmt(block, MirStmt::Assign(Place::local(local_idx), rvalue));
        block.unit()
    }
}
