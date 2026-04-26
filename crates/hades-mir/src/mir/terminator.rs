use hades_error::Span;
use hades_tokens::Name;

use super::{block::BlockId, local::LocalId, operand::Operand, place::Place};

/// Where a `SwitchInt` can jump.
#[derive(Debug, Clone)]
pub struct SwitchTargets {
    /// Discriminant values to test (parallel with `blocks`).
    pub values: Vec<u128>,
    /// Target block for each value (same length as `values`).
    pub blocks: Vec<BlockId>,
    /// Fallthrough block when no value matches.
    pub otherwise: BlockId,
}

impl SwitchTargets {
    /// Convenience constructor for boolean `if/else` branches.
    /// `then_block` is taken when the condition is true (1), `else_block` when false (0).
    pub fn bool_branch(then_block: BlockId, else_block: BlockId) -> Self {
        Self {
            values: vec![1],
            blocks: vec![then_block],
            otherwise: else_block,
        }
    }
}

/// How a `Call` terminator resolves the callee.
#[derive(Debug, Clone)]
pub enum CallTarget {
    /// A free function by name: `foo(args)`
    Function(Name),
    /// A method call: `receiver.method(args)`
    Method {
        receiver: Operand,
        method: Name,
    },
    /// A fully-qualified method call: `Type::method(args)`
    Qualified {
        ty: hades_ast::Types,
        method: Name,
    },
}

/// The kind of a block terminator.
#[derive(Debug, Clone)]
pub enum TerminatorKind {
    /// Unconditional branch to a single successor.
    Goto(BlockId),

    /// Conditional branch based on an integer discriminant.
    SwitchInt {
        discriminant: Operand,
        targets: SwitchTargets,
    },

    /// Return from the function (value is in `_0`).
    Return,

    /// Unreachable — used after diverging calls or at the end of a never-type branch.
    Unreachable,

    /// A function/method call that may jump to `destination` on return,
    /// or to `unwind` on panic (reserved for future use).
    Call {
        target: CallTarget,
        args: Vec<Operand>,
        /// Where to store the return value.
        destination: Place,
        /// Block to jump to after the call returns.
        successor: BlockId,
    },
}

/// A block terminator with its source span.
#[derive(Debug, Clone)]
pub struct Terminator {
    pub kind: TerminatorKind,
    pub span: Span,
}

impl Terminator {
    pub fn new(kind: TerminatorKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// The set of successor block ids for this terminator (used to build the CFG).
    pub fn successors(&self) -> Vec<BlockId> {
        match &self.kind {
            TerminatorKind::Goto(b) => vec![*b],
            TerminatorKind::SwitchInt { targets, .. } => {
                let mut succs: Vec<BlockId> = targets.blocks.clone();
                succs.push(targets.otherwise);
                succs.dedup();
                succs
            }
            TerminatorKind::Return | TerminatorKind::Unreachable => vec![],
            TerminatorKind::Call { successor, .. } => vec![*successor],
        }
    }
}

/// A phantom local used to hold the return-value of a `Call` before storing into its destination.
/// (Just a helper constant — `_0` is always the return slot.)
pub const RETURN_LOCAL: LocalId = LocalId(0);
