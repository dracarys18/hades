use std::collections::HashSet;

use super::{Lint, LintDiagnostic};
use hades_mir::mir::{
    func::MirFunction,
    local::Local,
    operand::Operand,
    place::{Place, PlaceElem},
    rvalue::Rvalue,
    stmt::StatementKind,
    terminator::{CallTarget, TerminatorKind},
};

pub struct UnusedVarLint;

type Used = HashSet<usize>;

impl Lint for UnusedVarLint {
    fn name(&self) -> &'static str {
        "unused_variable"
    }

    fn check_function(&self, func: &MirFunction) -> Vec<LintDiagnostic> {
        let mut used = Used::new();

        for bb in &func.guard.basic_blocks {
            for stmt in &bb.stmts {
                if let StatementKind::Assign(target, rvalue) = &stmt.kind {
                    mark_write(target, &mut used);
                    mark_rvalue(rvalue, &mut used);
                }
            }
            if let Some(terminator) = &bb.terminator {
                mark_terminator(&terminator.kind, &mut used);
            }
        }

        func.guard
            .locals
            .iter()
            .enumerate()
            .filter(|(local, data)| {
                is_user_var(*local, data, func.arg_count) && !used.contains(local)
            })
            .map(|(_, data)| {
                LintDiagnostic::warn(
                    self.name(),
                    format!("unused variable: `{}`", data.name.inner()),
                    data.name.span().clone(),
                )
            })
            .collect()
    }
}

fn is_user_var(local: usize, data: &Local, arg_count: usize) -> bool {
    local > arg_count && !data.name.inner().starts_with('_')
}

fn mark_rvalue(rvalue: &Rvalue, used: &mut Used) {
    match rvalue {
        Rvalue::Use(op) | Rvalue::UnaryOp(_, op) | Rvalue::Cast(op, _) | Rvalue::Repeat(op, _) => {
            mark_operand(op, used);
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            mark_operand(lhs, used);
            mark_operand(rhs, used);
        }
        Rvalue::Aggregate(_, operands) => operands.iter().for_each(|op| mark_operand(op, used)),
    }
}

fn mark_terminator(terminator: &TerminatorKind, used: &mut Used) {
    match terminator {
        TerminatorKind::SwitchInt { discriminant, .. } => mark_operand(discriminant, used),
        TerminatorKind::Call {
            target, args, dest, ..
        } => {
            if let CallTarget::Method { receiver, .. } = target {
                mark_operand(receiver, used);
            }
            args.iter().for_each(|op| mark_operand(op, used));
            mark_write(dest, used);
        }
        TerminatorKind::Goto(_) | TerminatorKind::Return | TerminatorKind::Unreachable => {}
    }
}

fn mark_operand(operand: &Operand, used: &mut Used) {
    match operand {
        Operand::Copy(place) | Operand::Ref(place) => mark_read(place, used),
        Operand::Const(_) => {}
    }
}

fn mark_read(place: &Place, used: &mut Used) {
    used.insert(place.local);
    mark_index_locals(&place.projection, used);
}

fn mark_write(place: &Place, used: &mut Used) {
    if place.projection.is_empty() {
        return;
    }
    used.insert(place.local);
    mark_index_locals(&place.projection, used);
}

fn mark_index_locals(projection: &[PlaceElem], used: &mut Used) {
    for elem in projection {
        if let PlaceElem::Index(local) = elem {
            used.insert(*local);
        }
    }
}
