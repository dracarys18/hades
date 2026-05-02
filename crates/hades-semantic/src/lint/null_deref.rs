use std::collections::HashMap;

use hades_ast::Types;
use hades_error::{Error, ErrorSeverity, Span};
use hades_mir::mir::block::BasicBlockData;
use hades_mir::mir::func::MirFunction;
use hades_mir::mir::operand::{MirConst, Operand};
use hades_mir::mir::rvalue::Rvalue;
use hades_mir::mir::stmt::StatementKind;
use hades_mir::mir::terminator::{Terminator, TerminatorKind};
use hades_tokens::Op;

use super::{Lint, LintDiagnostic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NullState {
    Unknown,
    NonNull,
    MaybeNull,
    DefinitelyNull,
}

impl NullState {
    fn join(self, other: NullState) -> NullState {
        use NullState::*;
        match (self, other) {
            (a, b) if a == b => a,
            (Unknown, x) | (x, Unknown) => x,
            (DefinitelyNull, NonNull) | (NonNull, DefinitelyNull) => MaybeNull,
            (MaybeNull, _) | (_, MaybeNull) => MaybeNull,
            _ => MaybeNull,
        }
    }
}

type State = HashMap<usize, NullState>;

pub struct NullDerefLint;

impl Lint for NullDerefLint {
    fn name(&self) -> &'static str {
        "null-deref"
    }

    fn check_function(&self, func: &MirFunction) -> Vec<LintDiagnostic> {
        let n_blocks = func.guard.basic_blocks.len();
        if n_blocks == 0 {
            return vec![];
        }

        let mut block_in: Vec<State> = vec![HashMap::new(); n_blocks];
        let mut block_out: Vec<State> = vec![HashMap::new(); n_blocks];

        for (idx, local) in func.guard.locals.iter().enumerate() {
            if matches!(local.typ, Types::Pointer(_)) {
                block_in[0].insert(idx, NullState::Unknown);
            }
        }

        let mut worklist: Vec<usize> = (0..n_blocks).collect();

        while let Some(block_idx) = worklist.pop() {
            let in_state = block_in[block_idx].clone();
            let out_state = transfer(&func.guard.basic_blocks[block_idx], in_state);

            if out_state != block_out[block_idx] {
                block_out[block_idx] = out_state.clone();
                for succ in &func.guard.basic_blocks[block_idx].successors {
                    let succ_idx = succ.0;
                    if join_into(&mut block_in[succ_idx], &out_state) {
                        worklist.push(succ_idx);
                    }
                }
            }
        }

        let mut diags = Vec::new();

        for (block_idx, block) in func.guard.basic_blocks.iter().enumerate() {
            let mut state = block_in[block_idx].clone();

            for stmt in &block.stmts {
                if let StatementKind::Assign(place, rvalue) = &stmt.kind {
                    check_deref_in_rvalue(rvalue, &state, &stmt.span, self.name(), &mut diags);
                    apply_stmt_to_state(place.local, rvalue, &mut state);
                }
            }

            if let Some(term) = &block.terminator {
                check_terminator_deref(term, &state, self.name(), &mut diags);
            }
        }

        diags
    }
}

fn transfer(block: &BasicBlockData, mut state: State) -> State {
    for stmt in &block.stmts {
        if let StatementKind::Assign(place, rvalue) = &stmt.kind {
            apply_stmt_to_state(place.local, rvalue, &mut state);
        }
    }
    state
}

fn apply_stmt_to_state(dest_local: usize, rvalue: &Rvalue, state: &mut State) {
    let new_state = match rvalue {
        Rvalue::Use(Operand::Const(MirConst::Null(_))) => NullState::DefinitelyNull,
        Rvalue::Use(Operand::Copy(p)) | Rvalue::Use(Operand::Ref(p)) => {
            if p.projection.is_empty() {
                state.get(&p.local).copied().unwrap_or(NullState::Unknown)
            } else {
                NullState::Unknown
            }
        }
        _ => NullState::NonNull,
    };

    if new_state != NullState::Unknown {
        state.insert(dest_local, new_state);
    }
}

fn join_into(dst: &mut State, src: &State) -> bool {
    let mut changed = false;
    for (&local, &src_state) in src {
        let entry = dst.entry(local).or_insert(NullState::Unknown);
        let joined = entry.join(src_state);
        if joined != *entry {
            *entry = joined;
            changed = true;
        }
    }
    changed
}

fn check_deref_in_rvalue(
    rvalue: &Rvalue,
    state: &State,
    span: &Span,
    lint_name: &'static str,
    diags: &mut Vec<LintDiagnostic>,
) {
    if let Rvalue::UnaryOp(Op::Deref, operand) = rvalue {
        let ptr_local = match operand {
            Operand::Copy(p) | Operand::Ref(p) => p.local,
            Operand::Const(_) => return,
        };
        match state.get(&ptr_local).copied().unwrap_or(NullState::Unknown) {
            NullState::DefinitelyNull => {
                diags.push(LintDiagnostic::error(
                    lint_name,
                    Error::new_with_span("".to_string(), span.clone())
                        .with_help("consider adding a null check before dereferencing".to_string()),
                ));
            }
            NullState::MaybeNull => {
                diags.push(LintDiagnostic::error(
                    lint_name,
                    Error::new_with_span(
                        "potential null pointer dereference: pointer may be null".to_string(),
                        span.clone(),
                    )
                    .with_severity(ErrorSeverity::Warning),
                ));
            }
            _ => {}
        }
    }
}

fn check_terminator_deref(
    term: &Terminator,
    state: &State,
    lint_name: &'static str,
    diags: &mut Vec<LintDiagnostic>,
) {
    if let TerminatorKind::Call { args, .. } = &term.kind {
        for op in args {
            if let Operand::Copy(p) | Operand::Ref(p) = op {
                if let Some(NullState::DefinitelyNull) = state.get(&p.local) {
                    diags.push(LintDiagnostic::error(
                        lint_name,
                        Error::new_with_span(
                            "null pointer passed to function call".to_string(),
                            term.span.clone(),
                        ),
                    ));
                }
            }
        }
    }
}
