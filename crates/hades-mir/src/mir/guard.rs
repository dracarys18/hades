use std::collections::HashMap;

use hades_tokens::Ident;

use crate::BasicBlock;
use crate::mir::block::BasicBlockData;
use crate::mir::local::Local;
use crate::mir::stmt::MirStmt;
use crate::mir::terminator::Terminator;

pub struct Guard {
    pub locals: Vec<Local>,
    pub local_map: HashMap<Ident, usize>,
    pub basic_blocks: Vec<BasicBlockData>,
}

impl Guard {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            local_map: HashMap::new(),
            basic_blocks: Vec::new(),
        }
    }

    pub fn add_local(&mut self, local: Local) -> usize {
        let idx = self.locals.len();
        self.local_map.insert(local.name().clone(), idx);
        self.locals.push(local);
        idx
    }

    pub fn start_new_block(&mut self) -> BasicBlock {
        let idx = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlockData::new());
        BasicBlock(idx)
    }

    pub fn push_stmt(&mut self, block: BasicBlock, stmt: MirStmt) {
        self.basic_blocks[block.0].push(stmt);
    }

    pub fn terminate(&mut self, block: BasicBlock, t: Terminator) {
        self.basic_blocks[block.0].terminator = Some(t);
    }

    pub fn lookup_local(&self, name: &Ident) -> usize {
        *self.local_map.get(name).expect("undeclared local")
    }
}
