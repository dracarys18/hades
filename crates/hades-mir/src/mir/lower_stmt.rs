use hades_ast::{
    TypedBlock, TypedBreak, TypedContinue, TypedDefer, TypedFor, TypedIf, TypedLet, TypedReturn,
    TypedStmt, TypedWhile,
};

use crate::{
    ToMir,
    builder::MirBuilder,
    mir::{
        expr::lower_assign,
        place::Place,
        rvalue::Rvalue,
        stmt::Statement,
        terminator::{Terminator, TerminatorKind, SwitchTargets},
    },
};

// ── TypedBlock ────────────────────────────────────────────────────────────────

impl ToMir for TypedBlock {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        for stmt in self.stmts.iter() {
            stmt.to_mir(builder);
            // If the current block is now terminated (e.g. return/break/continue),
            // remaining statements are unreachable — stop.
            if builder.is_terminated() {
                break;
            }
        }
    }
}

// ── TypedStmt ─────────────────────────────────────────────────────────────────

impl ToMir for TypedStmt {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        match self {
            TypedStmt::Let(s) => s.to_mir(builder),
            TypedStmt::TypedExpr(e) => {
                // Expression as a statement — evaluate for side effects, discard result.
                e.expr.to_mir(builder);
            }
            TypedStmt::If(s) => s.to_mir(builder),
            TypedStmt::While(s) => s.to_mir(builder),
            TypedStmt::For(s) => s.to_mir(builder),
            TypedStmt::Block(b) => b.to_mir(builder),
            TypedStmt::Return(s) => s.to_mir(builder),
            TypedStmt::Continue(s) => s.to_mir(builder),
            TypedStmt::Break(s) => s.to_mir(builder),
            TypedStmt::Defer(s) => s.to_mir(builder),
            // FuncDef / StructDef / ModuleDecl / Import handled at module level, not in CFG.
            TypedStmt::FuncDef(_)
            | TypedStmt::StructDef(_)
            | TypedStmt::ModuleDecl(_)
            | TypedStmt::Import(_) => {}
        }
    }
}

// ── TypedLet ──────────────────────────────────────────────────────────────────

impl ToMir for TypedLet {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let span = self.span.clone();
        let operand = self.value.expr.to_mir(builder);
        let local = builder.new_named_local(self.name.clone(), self.typ.clone(), span.clone());
        builder.emit(Statement::assign(
            Place::local(local),
            Rvalue::Use(operand),
            span,
        ));
    }
}

// ── TypedIf ───────────────────────────────────────────────────────────────────

impl ToMir for TypedIf {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let span = self.span.clone();
        let cond = self.cond.expr.to_mir(builder);

        let then_block = builder.new_block();
        let else_block = builder.new_block();
        let merge_block = builder.new_block();

        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond,
                targets: SwitchTargets::bool_branch(then_block, else_block),
            },
            span.clone(),
        ));

        // Then branch.
        builder.switch_to(then_block);
        self.then_branch.to_mir(builder);
        if !builder.is_terminated() {
            builder.terminate(Terminator::new(TerminatorKind::Goto(merge_block), span.clone()));
        }

        // Else branch.
        builder.switch_to(else_block);
        if let Some(else_b) = &self.else_branch {
            else_b.to_mir(builder);
        }
        if !builder.is_terminated() {
            builder.terminate(Terminator::new(TerminatorKind::Goto(merge_block), span.clone()));
        }

        builder.switch_to(merge_block);
    }
}

// ── TypedWhile ────────────────────────────────────────────────────────────────

impl ToMir for TypedWhile {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let span = self.span.clone();

        let header_block = builder.new_block();
        let body_block = builder.new_block();
        let exit_block = builder.new_block();

        builder.terminate(Terminator::new(TerminatorKind::Goto(header_block), span.clone()));

        builder.switch_to(header_block);
        let cond = self.cond.to_mir(builder);
        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond,
                targets: SwitchTargets::bool_branch(body_block, exit_block),
            },
            span.clone(),
        ));

        builder.switch_to(body_block);
        builder.push_loop(header_block, exit_block);
        self.body.to_mir(builder);
        builder.pop_loop();
        if !builder.is_terminated() {
            builder.terminate(Terminator::new(TerminatorKind::Goto(header_block), span.clone()));
        }

        builder.switch_to(exit_block);
    }
}

// ── TypedFor ──────────────────────────────────────────────────────────────────

impl ToMir for TypedFor {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let span = self.span.clone();

        // Init.
        self.init.to_mir(builder);

        let header_block = builder.new_block();
        let body_block = builder.new_block();
        let update_block = builder.new_block();
        let exit_block = builder.new_block();

        builder.terminate(Terminator::new(TerminatorKind::Goto(header_block), span.clone()));

        // Header: condition (TypedBinaryExpr wrapped as TypedExpr).
        builder.switch_to(header_block);
        let cond_op = hades_ast::TypedExpr::Binary(self.cond.clone()).to_mir(builder);
        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond_op,
                targets: SwitchTargets::bool_branch(body_block, exit_block),
            },
            span.clone(),
        ));

        // Body.
        builder.switch_to(body_block);
        builder.push_loop(update_block, exit_block);
        self.body.to_mir(builder);
        builder.pop_loop();
        if !builder.is_terminated() {
            builder.terminate(Terminator::new(TerminatorKind::Goto(update_block), span.clone()));
        }

        // Update.
        builder.switch_to(update_block);
        lower_assign(&self.update, builder, span.clone());
        builder.terminate(Terminator::new(TerminatorKind::Goto(header_block), span.clone()));

        builder.switch_to(exit_block);
    }
}

// ── TypedReturn ───────────────────────────────────────────────────────────────

impl ToMir for TypedReturn {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let span = self.span.clone();

        // Inline all pending defer blocks (LIFO) before the return instruction.
        // Re-lower each deferred AST block directly into the current block.
        let deferred = builder.deferred_blocks();
        for block in deferred {
            block.to_mir(builder);
        }

        // Write return value into `_0`.
        if let Some(ret_expr) = &self.expr {
            let operand = ret_expr.expr.to_mir(builder);
            builder.emit(Statement::assign(
                Place::local(crate::mir::terminator::RETURN_LOCAL),
                Rvalue::Use(operand),
                span.clone(),
            ));
        }

        builder.terminate(Terminator::new(TerminatorKind::Return, span));
    }
}

// ── TypedContinue ─────────────────────────────────────────────────────────────

impl ToMir for TypedContinue {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let loop_ctx = builder.current_loop().expect("continue outside loop").clone();
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(loop_ctx.continue_block),
            self.span.clone(),
        ));
    }
}

// ── TypedBreak ────────────────────────────────────────────────────────────────

impl ToMir for TypedBreak {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        let loop_ctx = builder.current_loop().expect("break outside loop").clone();
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(loop_ctx.break_block),
            self.span.clone(),
        ));
    }
}

// ── TypedDefer ────────────────────────────────────────────────────────────────

impl ToMir for TypedDefer {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder) {
        // Push the deferred AST block onto the defer stack.
        // It will be re-lowered at each Return site (LIFO order).
        builder.push_defer(self.stmt.clone(), self.span.clone());
    }
}
