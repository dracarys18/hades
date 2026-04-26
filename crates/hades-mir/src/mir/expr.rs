use hades_ast::{
    TypedArrayLiteral, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr, TypedExpr,
    TypedFieldAccess, TypedValue,
};
use hades_error::Span;
use hades_tokens::Op;

use crate::{
    ToMir,
    builder::MirBuilder,
    mir::{
        local::LocalId,
        operand::{MirConst, Operand},
        place::{Place, PlaceElem},
        rvalue::{AggregateKind, Rvalue},
        stmt::Statement,
        terminator::{CallTarget, Terminator, TerminatorKind},
    },
};

/// Lower a `TypedExpr` to an `Operand` that names the result.
/// Constants produce an inline `Operand::Const`.
/// All other expressions materialise into a fresh temporary.
impl ToMir for TypedExpr {
    type Output = Operand;

    fn to_mir(&self, builder: &mut MirBuilder) -> Operand {
        let span = dummy_span();
        match self {
            TypedExpr::Value(v) => lower_value(v, builder, span),

            TypedExpr::Null(ty) => Operand::Const(MirConst::Null(ty.clone())),

            TypedExpr::Ident { ident, typ } => {
                if let Some(local) = builder.lookup_local(ident) {
                    Operand::Copy(Place::local(local))
                } else {
                    let temp = builder.new_local(typ.clone(), span);
                    Operand::Copy(Place::local(temp))
                }
            }

            TypedExpr::Binary(TypedBinaryExpr { left, op, right, typ }) => {
                let lhs = left.to_mir(builder);
                let rhs = right.to_mir(builder);
                let temp = builder.new_local(typ.clone(), span.clone());
                builder.emit(Statement::assign(
                    Place::local(temp),
                    Rvalue::BinaryOp(op.clone(), lhs, rhs),
                    span,
                ));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::Unary { op, expr, typ } => {
                let operand = expr.to_mir(builder);
                let temp = builder.new_local(typ.clone(), span.clone());
                // `&x` is pointer-taking — expressed as Operand::Ref, not a unary op.
                let rvalue = if *op == Op::Ref {
                    let place = match operand {
                        Operand::Copy(p) => p,
                        // Ref of a ref is unusual but handle gracefully.
                        Operand::Ref(p) => p,
                        Operand::Const(_) => {
                            // Cannot take address of a constant directly — materialise first.
                            let tmp = builder.new_local(expr.get_type(), span.clone());
                            builder.emit(Statement::assign(
                                Place::local(tmp),
                                Rvalue::Use(operand),
                                span.clone(),
                            ));
                            Place::local(tmp)
                        }
                    };
                    Rvalue::Use(Operand::Ref(place))
                } else {
                    Rvalue::UnaryOp(op.clone(), operand)
                };
                builder.emit(Statement::assign(Place::local(temp), rvalue, span));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::As(cast) => {
                let operand = cast.expr.to_mir(builder);
                let temp = builder.new_local(cast.target_type.clone(), span.clone());
                builder.emit(Statement::assign(
                    Place::local(temp),
                    Rvalue::Cast(operand, cast.target_type.clone()),
                    span,
                ));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::FieldAccess(TypedFieldAccess {
                expr,
                field,
                struct_type,
                field_type,
            }) => {
                let base_type = expr.get_type();
                let place = if let TypedExpr::Unary { op: Op::Deref, expr: ptr_expr, .. } = expr.as_ref() {
                    // `(*ptr).field` — build [Deref, Field] on the pointer local.
                    let ptr_op = ptr_expr.to_mir(builder);
                    let ptr_ty = ptr_expr.get_type();
                    let ptr_local = materialize_operand(ptr_op, ptr_ty, builder, span.clone());
                    Place {
                        local: ptr_local,
                        projection: vec![
                            PlaceElem::Deref,
                            PlaceElem::Field { name: field.clone(), index: 0, ty: field_type.clone() },
                        ],
                    }
                } else if matches!(base_type, hades_ast::Types::Pointer(_)) {
                    // Auto-deref: `ptr.field` where ptr: &Struct.
                    let ptr_op = expr.to_mir(builder);
                    let ptr_local = materialize_operand(ptr_op, base_type, builder, span.clone());
                    Place {
                        local: ptr_local,
                        projection: vec![
                            PlaceElem::Deref,
                            PlaceElem::Field { name: field.clone(), index: 0, ty: field_type.clone() },
                        ],
                    }
                } else {
                    let base_operand = expr.to_mir(builder);
                    let base_local = materialize_operand(base_operand, struct_type.clone(), builder, span.clone());
                    Place {
                        local: base_local,
                        projection: vec![PlaceElem::Field {
                            name: field.clone(),
                            index: 0,
                            ty: field_type.clone(),
                        }],
                    }
                };
                let temp = builder.new_local(field_type.clone(), span.clone());
                builder.emit(Statement::assign(
                    Place::local(temp),
                    Rvalue::Use(Operand::Copy(place)),
                    span,
                ));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::ArrayIndex(arr_idx) => {
                let elem_ty = arr_idx.typ.get_array_elem_type();
                let base_operand = arr_idx.expr.to_mir(builder);
                let base_local =
                    materialize_operand(base_operand, arr_idx.typ.clone(), builder, span.clone());
                let idx_operand = arr_idx.index.to_mir(builder);
                let idx_local =
                    materialize_operand(idx_operand, hades_ast::Types::Int, builder, span.clone());
                let place = Place::with_index(base_local, idx_local);
                let temp = builder.new_local(elem_ty, span.clone());
                builder.emit(Statement::assign(
                    Place::local(temp),
                    Rvalue::Use(Operand::Copy(place)),
                    span,
                ));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::StructInit { name, fields, types, .. } => {
                let field_operands: Vec<Operand> =
                    fields.values().map(|e| e.to_mir(builder)).collect();
                let temp = builder.new_local(types.clone(), span.clone());
                builder.emit(Statement::assign(
                    Place::local(temp),
                    Rvalue::Aggregate(
                        AggregateKind::Struct(name.inner().to_string()),
                        field_operands,
                    ),
                    span,
                ));
                Operand::Copy(Place::local(temp))
            }

            TypedExpr::Assign(assign) => {
                lower_assign(assign, builder, span);
                Operand::Const(MirConst::Int(0))
            }

            TypedExpr::Call { func, args, receiver, typ } => {
                let arg_operands: Vec<Operand> = args.iter().map(|a| a.to_mir(builder)).collect();
                let call_target = if let Some(recv) = receiver {
                    let recv_op = recv.to_mir(builder);
                    CallTarget::Method {
                        receiver: recv_op,
                        method: func.clone(),
                    }
                } else {
                    CallTarget::Function(func.clone())
                };
                let dest_temp = builder.new_local(typ.clone(), span.clone());
                let successor = builder.new_block();
                builder.terminate(Terminator::new(
                    TerminatorKind::Call {
                        target: call_target,
                        args: arg_operands,
                        destination: Place::local(dest_temp),
                        successor,
                    },
                    span,
                ));
                builder.switch_to(successor);
                Operand::Copy(Place::local(dest_temp))
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn lower_value(v: &TypedValue, builder: &mut MirBuilder, span: Span) -> Operand {
    match v {
        TypedValue::Number(n) => Operand::Const(MirConst::Int(*n)),
        TypedValue::Float(f) => Operand::Const(MirConst::Float(*f)),
        TypedValue::Boolean(b) => Operand::Const(MirConst::Bool(*b)),
        TypedValue::String(s) => Operand::Const(MirConst::String(s.clone())),
        TypedValue::Char(c) => Operand::Const(MirConst::Char(*c)),
        TypedValue::Array(arr) => lower_array(arr, builder, span),
    }
}

fn lower_array(arr: &TypedArrayLiteral, builder: &mut MirBuilder, span: Span) -> Operand {
    let elem_operands: Vec<Operand> = if let Some(fill) = &arr.fill {
        (0..arr.size).map(|_| fill.to_mir(builder)).collect()
    } else {
        arr.elements.iter().map(|e| e.to_mir(builder)).collect()
    };
    let arr_ty = hades_ast::Types::Array(arr.elem_typ.array_type(arr.size));
    let temp = builder.new_local(arr_ty, span.clone());
    builder.emit(Statement::assign(
        Place::local(temp),
        Rvalue::Aggregate(AggregateKind::Array(arr.elem_typ.clone()), elem_operands),
        span,
    ));
    Operand::Copy(Place::local(temp))
}

/// Lower an assignment expression into the current block.
pub(crate) fn lower_assign(assign: &TypedAssignExpr, builder: &mut MirBuilder, span: Span) {
    let val_operand = assign.value.to_mir(builder);
    let rvalue = match assign.op {
        Op::PlusEqual => {
            let target_op = lower_assign_target_read(&assign.target, builder, span.clone());
            Rvalue::BinaryOp(Op::Plus, target_op, val_operand)
        }
        Op::MinusEqual => {
            let target_op = lower_assign_target_read(&assign.target, builder, span.clone());
            Rvalue::BinaryOp(Op::Minus, target_op, val_operand)
        }
        _ => Rvalue::Use(val_operand),
    };
    let place = lower_assign_target_place(&assign.target, builder, span.clone());
    builder.emit(Statement::assign(place, rvalue, span));
}

fn lower_assign_target_place(
    target: &TypedAssignTarget,
    builder: &mut MirBuilder,
    span: Span,
) -> Place {
    match target {
        TypedAssignTarget::Ident(ident) => {
            let local = builder
                .lookup_local(ident)
                .unwrap_or_else(|| builder.new_local(hades_ast::Types::Void, span));
            Place::local(local)
        }
        TypedAssignTarget::FieldAccess(fa) => {
            // If the base expression is a pointer deref (`*ptr`), build a
            // [Deref, Field] projection on the pointer local instead of
            // materialising a copy of the dereffed struct value.
            if let TypedExpr::Unary { op: Op::Deref, expr: ptr_expr, .. } = fa.expr.as_ref() {
                let ptr_op = ptr_expr.to_mir(builder);
                let ptr_ty = ptr_expr.get_type();
                let ptr_local = materialize_operand(ptr_op, ptr_ty, builder, span);
                Place {
                    local: ptr_local,
                    projection: vec![
                        PlaceElem::Deref,
                        PlaceElem::Field {
                            name: fa.field.clone(),
                            index: 0,
                            ty: fa.field_type.clone(),
                        },
                    ],
                }
            } else if matches!(fa.expr.get_type(), hades_ast::Types::Pointer(_)) {
                // Auto-deref: `ptr.field` where ptr: &Struct — same as `(*ptr).field`.
                let ptr_op = fa.expr.to_mir(builder);
                let ptr_ty = fa.expr.get_type();
                let ptr_local = materialize_operand(ptr_op, ptr_ty, builder, span);
                Place {
                    local: ptr_local,
                    projection: vec![
                        PlaceElem::Deref,
                        PlaceElem::Field {
                            name: fa.field.clone(),
                            index: 0,
                            ty: fa.field_type.clone(),
                        },
                    ],
                }
            } else {
                let base_op = fa.expr.to_mir(builder);
                let base_local = materialize_operand(base_op, fa.struct_type.clone(), builder, span);
                Place::with_field(base_local, fa.field.clone(), 0, fa.field_type.clone())
            }
        }
        TypedAssignTarget::ArrayIndex(ai) => {
            let base_op = ai.expr.to_mir(builder);
            let base_local = materialize_operand(base_op, ai.typ.clone(), builder, span.clone());
            let idx_op = ai.index.to_mir(builder);
            let idx_local = materialize_operand(idx_op, hades_ast::Types::Int, builder, span);
            Place::with_index(base_local, idx_local)
        }
        TypedAssignTarget::Deref(expr) => {
            let ptr_op = expr.to_mir(builder);
            let ptr_ty = expr.get_type();
            let ptr_local = materialize_operand(ptr_op, ptr_ty, builder, span);
            Place::with_deref(ptr_local)
        }
    }
}

fn lower_assign_target_read(
    target: &TypedAssignTarget,
    builder: &mut MirBuilder,
    span: Span,
) -> Operand {
    let place = lower_assign_target_place(target, builder, span);
    Operand::Copy(place)
}

/// If the operand is already a bare-local, return its `LocalId`.
/// Otherwise materialise it into a fresh temporary and return that id.
pub(crate) fn materialize_operand(
    op: Operand,
    ty: hades_ast::Types,
    builder: &mut MirBuilder,
    span: Span,
) -> LocalId {
    match op {
        Operand::Copy(Place { local, projection }) if projection.is_empty() => local,
        Operand::Ref(Place { local, projection }) if projection.is_empty() => local,
        other => {
            let temp = builder.new_local(ty, span.clone());
            builder.emit(Statement::assign(
                Place::local(temp),
                Rvalue::Use(other),
                span,
            ));
            temp
        }
    }
}

fn dummy_span() -> Span {
    Span::default()
}
