mod mir;

use hades_module::TypedModule;
use mir::builder::MirBuilder;

pub const RETURN_LOCAL: usize = 0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BasicBlock(pub usize);

#[must_use]
pub struct BlockAnd<T>(pub BasicBlock, pub T);

pub trait BlockAndExt {
    fn and<T>(self, v: T) -> BlockAnd<T>;
    fn unit(self) -> BlockAnd<()>;
}

impl BlockAndExt for BasicBlock {
    fn and<T>(self, v: T) -> BlockAnd<T> {
        BlockAnd(self, v)
    }
    fn unit(self) -> BlockAnd<()> {
        BlockAnd(self, ())
    }
}

#[macro_export]
macro_rules! unpack {
    ($block:ident = $expr:expr) => {{
        let $crate::BlockAnd(b, v) = $expr;
        $block = b;
        v
    }};
}

pub trait ToMir {
    type Output;
    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<Self::Output>;
}

pub fn lower(module: &TypedModule) {
    let mut builder = MirBuilder::new(&module.ctx);
    builder.build();
}
