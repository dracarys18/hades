use std::collections::HashMap;

use hades_ast::Types;
use hades_error::{Error, ErrorSeverity, Span};
use hades_mir::mir::func::MirFunction;
use hades_mir::mir::local::Local;
use hades_mir::mir::operand::{MirConst, Operand};
use hades_mir::mir::place::PlaceElem;
use hades_mir::mir::rvalue::Rvalue;
use hades_mir::mir::stmt::StatementKind;

use super::{Lint, LintDiagnostic};

pub struct ArrayBoundsLint;

impl Lint for ArrayBoundsLint {
    fn name(&self) -> &'static str {
        "array-out-of-bounds"
    }

    fn check_function(&self, func: &MirFunction) -> Vec<LintDiagnostic> {
        let mut diags = Vec::new();

        for block in &func.guard.basic_blocks {
            let mut const_map: HashMap<usize, i64> = HashMap::new();

            for stmt in &block.stmts {
                if let StatementKind::Assign(place, rvalue) = &stmt.kind {
                    if place.projection.is_empty()
                        && let Rvalue::Use(Operand::Const(MirConst::Int(n))) = rvalue.as_ref()
                    {
                        const_map.insert(place.local, *n);
                    }

                    check_projections(
                        place.local,
                        &place.projection,
                        &const_map,
                        &func.guard.locals,
                        &stmt.span,
                        self.name(),
                        &mut diags,
                    );

                    check_rvalue_projections(
                        rvalue,
                        &const_map,
                        &func.guard.locals,
                        &stmt.span,
                        self.name(),
                        &mut diags,
                    );
                }
            }
        }

        diags
    }
}

fn array_len(locals: &[Local], local_idx: usize) -> Option<usize> {
    let typ = &locals.get(local_idx)?.typ;
    match typ {
        Types::Array(_) => Some(typ.get_array_size()),
        _ => None,
    }
}

fn check_projections(
    base_local: usize,
    projections: &[PlaceElem],
    const_map: &HashMap<usize, i64>,
    locals: &[Local],
    span: &Span,
    lint_name: &'static str,
    diags: &mut Vec<LintDiagnostic>,
) {
    for elem in projections {
        if let PlaceElem::Index(idx_local) = elem {
            let len = array_len(locals, base_local);
            match const_map.get(idx_local) {
                Some(&idx_val) => {
                    if let Some(len) = len
                        && (idx_val < 0 || idx_val as usize >= len)
                    {
                        diags.push(LintDiagnostic::error(
                            lint_name,
                            Error::new_with_span(
                                format!(
                                    "index out of bounds: index is {idx_val}, but length is {len}"
                                ),
                                span.clone(),
                            ),
                        ));
                    }
                }
                None => {
                    diags.push(LintDiagnostic::error(
                        lint_name,
                        Error::new_with_span(
                            "array index is not a compile-time constant; bounds cannot be verified"
                                .to_string(),
                            span.clone(),
                        )
                        .with_severity(ErrorSeverity::Warning),
                    ));
                }
            }
        }
    }
}

fn check_rvalue_projections(
    rvalue: &Rvalue,
    const_map: &HashMap<usize, i64>,
    locals: &[Local],
    span: &Span,
    lint_name: &'static str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let operands: Vec<&Operand> = match rvalue {
        Rvalue::Use(op) => vec![op],
        Rvalue::BinaryOp(_, a, b) => vec![a, b],
        Rvalue::UnaryOp(_, a) => vec![a],
        Rvalue::Cast(op, _) => vec![op],
        Rvalue::Aggregate(_, ops) => ops.iter().collect(),
        Rvalue::Repeat(op, n) => std::iter::repeat_n(op, *n).collect(),
    };

    for op in operands {
        if let Operand::Copy(place) | Operand::Ref(place) = op {
            check_projections(
                place.local,
                &place.projection,
                const_map,
                locals,
                span,
                lint_name,
                diags,
            );
        }
    }
}
