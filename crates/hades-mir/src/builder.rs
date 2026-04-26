use hades_ast::{TypedBlock, Types};
use hades_error::Span;
use hades_tokens::Ident;
use indexmap::IndexMap;

use crate::mir::{
    block::{BasicBlock, BlockId},
    cfg::Cfg,
    local::{LocalDecl, LocalId},
    stmt::Statement,
    terminator::Terminator,
};

/// Loop context: the blocks that `break` and `continue` should jump to.
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub continue_block: BlockId,
    pub break_block: BlockId,
}

/// Stack of pending defer blocks to be inlined at each `Return` site.
/// Each entry stores the AST block so it can be re-lowered at every return point.
#[derive(Debug, Clone)]
pub struct DeferEntry {
    pub block: TypedBlock,
    pub span: Span,
}

/// Pure infrastructure for building a MIR function's CFG.
/// No lowering logic lives here — all lowering is in `ToMir` impls.
pub struct MirBuilder {
    /// All basic blocks accumulated so far.
    blocks: Vec<BasicBlock>,

    /// Index of the currently-open block.
    current: BlockId,

    /// Counter for the next `LocalId` to allocate.
    next_local: u32,

    /// All local declarations (indexed by `LocalId`).
    pub locals: Vec<LocalDecl>,

    /// Maps variable names (from `let` statements) to their `LocalId`.
    pub name_map: IndexMap<Ident, LocalId>,

    /// Stack of active loop contexts for `break`/`continue`.
    pub loop_stack: Vec<LoopContext>,

    /// Stack of pending defer blocks; inlined at every `Return` site.
    pub defer_stack: Vec<DeferEntry>,
}

impl MirBuilder {
    /// Create a builder pre-seeded with `_0` as the return slot.
    pub fn new(return_type: Types, return_span: Span) -> Self {
        let return_decl = LocalDecl::new(return_type, return_span);
        let entry = BasicBlock::new(BlockId::new(0));
        Self {
            blocks: vec![entry],
            current: BlockId::new(0),
            next_local: 1, // _0 already allocated below
            locals: vec![return_decl],
            name_map: IndexMap::new(),
            loop_stack: vec![],
            defer_stack: vec![],
        }
    }

    // ── Locals ────────────────────────────────────────────────────────────

    /// Allocate a new anonymous temporary local.
    pub fn new_local(&mut self, ty: Types, span: Span) -> LocalId {
        let id = LocalId::new(self.next_local);
        self.next_local += 1;
        self.locals.push(LocalDecl::new(ty, span));
        id
    }

    /// Allocate a named local (from `let name: T`) and register it in `name_map`.
    pub fn new_named_local(&mut self, name: Ident, ty: Types, span: Span) -> LocalId {
        let id = self.new_local(ty, span);
        self.name_map.insert(name, id);
        id
    }

    /// Look up a named variable.
    pub fn lookup_local(&self, name: &Ident) -> Option<LocalId> {
        self.name_map.get(name).copied()
    }

    // ── Blocks ────────────────────────────────────────────────────────────

    /// Allocate a new basic block (not yet current).
    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId::new(self.blocks.len() as u32);
        self.blocks.push(BasicBlock::new(id));
        id
    }

    /// The id of the block currently being built.
    pub fn current_block(&self) -> BlockId {
        self.current
    }

    /// Switch the current block to `id`.
    pub fn switch_to(&mut self, id: BlockId) {
        self.current = id;
    }

    /// Is the current block already terminated?
    pub fn is_terminated(&self) -> bool {
        self.blocks[self.current.index()].is_terminated()
    }

    /// Append a statement to the current block.
    pub fn emit(&mut self, stmt: Statement) {
        debug_assert!(
            !self.is_terminated(),
            "emitting into already-terminated block {}",
            self.current
        );
        self.blocks[self.current.index()].stmts.push(stmt);
    }

    /// Terminate the current block.
    pub fn terminate(&mut self, t: Terminator) {
        self.blocks[self.current.index()].terminate(t);
    }

    // ── Loops ─────────────────────────────────────────────────────────────

    pub fn push_loop(&mut self, continue_block: BlockId, break_block: BlockId) {
        self.loop_stack.push(LoopContext {
            continue_block,
            break_block,
        });
    }

    pub fn pop_loop(&mut self) {
        self.loop_stack.pop().expect("pop_loop: stack is empty");
    }

    pub fn current_loop(&self) -> Option<&LoopContext> {
        self.loop_stack.last()
    }

    // ── Defers ────────────────────────────────────────────────────────────

    pub fn push_defer(&mut self, block: TypedBlock, span: Span) {
        self.defer_stack.push(DeferEntry { block, span });
    }

    pub fn pop_defer(&mut self) {
        self.defer_stack.pop().expect("pop_defer: stack is empty");
    }

    /// Clone the deferred blocks in LIFO order (last-in, first-out at return).
    pub fn deferred_blocks(&self) -> Vec<TypedBlock> {
        self.defer_stack
            .iter()
            .rev()
            .map(|d| d.block.clone())
            .collect()
    }

    // ── Finish ────────────────────────────────────────────────────────────

    /// Consume the builder and produce a finished `Cfg`.
    pub fn finish(self) -> (Cfg, Vec<LocalDecl>) {
        let cfg = Cfg::finish(BlockId::new(0), self.blocks);
        (cfg, self.locals)
    }
}
