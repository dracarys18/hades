pub mod mir;

use hades_ast::{TypedFieldKind, TypedStmt};
use hades_module::TypedModule;
use mir::builder::MirBuilder;
use mir::func::MirFunction;
use mir::module::MirModule;

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

pub fn lower(module: TypedModule) -> MirModule {
    let mut dummy = MirBuilder::new();
    let mut functions: Vec<MirFunction> = vec![];

    for stmt in module.program.iter() {
        match stmt {
            TypedStmt::FuncDef(func_def) => {
                let BlockAnd(_, maybe_fn) = func_def.to_mir(&mut dummy, BasicBlock(0));
                if let Some(mir_fn) = maybe_fn {
                    functions.push(mir_fn);
                }
            }
            TypedStmt::StructDef(struct_def) => {
                for (_field_name, field_kind) in &struct_def.fields {
                    if let TypedFieldKind::Func(method) = field_kind {
                        let BlockAnd(_, maybe_fn) = method.to_mir(&mut dummy, BasicBlock(0));
                        if let Some(mir_fn) = maybe_fn {
                            functions.push(mir_fn);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    MirModule {
        path: module.path,
        functions,
    }
}
