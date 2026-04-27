use crate::mir::{block::BasicBlock, local::Local};

pub(crate) struct Guard {
    locals: Vec<Local>,
    basic_blocks: Vec<BasicBlock>,
}

impl Guard {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            basic_blocks: Vec::new(),
        }
    }

    pub fn add_local(&mut self, local: Local) {
        self.locals.push(local);
    }

    pub fn add_basic_block(&mut self, block: BasicBlock) {
        self.basic_blocks.push(block);
    }
}
