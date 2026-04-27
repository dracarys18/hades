use crate::mir::{builder::MirBuilder, stmt::MirStmt};
use hades_ast::TypedBlock;

use crate::{ToMir, id::Id};

pub(crate) struct BasicBlock {
    id: Id,
    statements: Vec<MirStmt>,
    terminator: Option<MirTerminator>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self { id: Id::new() }
    }
}

impl ToMir for TypedBlock {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) -> Self::Output {
        for stmt in &self.stmts {
            stmt.to_mir(builder);
        }
    }
}
