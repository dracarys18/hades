use hades_ast::TypedStmt;

use crate::{ToMir, mir::builder::MirBuilder};

pub enum MirStmt {}

impl ToMir for TypedStmt {
    type Output = MirStmt;

    fn to_mir(&self, _builder: &mut MirBuilder) -> Self::Output {
        unimplemented!()
    }
}
