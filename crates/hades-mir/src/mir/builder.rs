use hades_ast::{CompilerContext, Types};

use crate::mir::{block::BasicBlock, guard::Guard, local::Local};

pub(crate) struct MirBuilder<'ctx> {
    ctx: &'ctx CompilerContext,
    current_guard: Option<Guard>,
}

impl<'ctx> MirBuilder<'ctx> {
    pub fn new(ctx: &'ctx CompilerContext) -> Self {
        Self {
            ctx,
            current_guard: None,
        }
    }

    pub fn enter_guard(&mut self) {
        self.current_guard = Some(Guard::new());
    }

    pub fn exit_guard(&mut self) -> Guard {
        self.current_guard.take().expect("No active guard to exit")
    }

    pub fn build_local(&mut self, typ: Types) {
        let local = Local::new(typ);
        let guard = self.current_guard.as_mut().expect("No active guard");
        guard.add_local(local);
    }

    pub fn build_basic_block(&mut self) {
        let block = BasicBlock::new();
        let guard = self.current_guard.as_mut().expect("No active guard");
        guard.add_basic_block(block);
    }

    pub fn build(&mut self) {}
}
