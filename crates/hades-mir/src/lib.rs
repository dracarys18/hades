mod id;
mod mir;

use hades_ast::TypedStmt;
use hades_module::{ModuleSignatures, TypedModule};

use mir::builder::MirBuilder;

pub trait ToMir {
    type Output;
    fn to_mir(&self, builder: &mut MirBuilder) -> Self::Output;
}

pub fn lower(module: &TypedModule) {
    let mut build = MirBuilder::new(&module.ctx);
    build.build()
}
