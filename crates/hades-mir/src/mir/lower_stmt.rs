use hades_ast::{
    TypedBlock, TypedBreak, TypedContinue, TypedDefer, TypedFor, TypedIf, TypedLet, TypedReturn,
    TypedWhile,
};

use crate::mir::builder::MirBuilder;
use crate::mir::expr::emit_temp;
use crate::mir::place::Place;
use crate::mir::stmt::Statement;
use crate::mir::terminator::{RETURN_LOCAL, SwitchTargets, Terminator, TerminatorKind};
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

impl ToMir for TypedBlock {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        for stmt in &self.stmts {
            unpack!(block = stmt.to_mir(builder, block));
            if builder.is_terminated() {
                break;
            }
        }
        block.unit()
    }
}

impl ToMir for TypedLet {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();
        let rvalue = unpack!(block = self.value.expr.to_mir(builder, block));
        let local = builder.build_local(self.name.clone(), self.typ.clone());
        builder.push_stmt(block, Statement::assign(Place::local(local), rvalue, span));
        block.unit()
    }
}

impl ToMir for TypedReturn {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();

        let deferred = builder.deferred_stmts();
        for stmt in deferred {
            builder.push_stmt(block, stmt);
        }

        if let Some(ret_expr) = &self.expr {
            let rvalue = unpack!(block = ret_expr.expr.to_mir(builder, block));
            builder.push_stmt(
                block,
                Statement::assign(Place::local(RETURN_LOCAL), rvalue, span.clone()),
            );
        }

        builder.switch_to(block);
        builder.terminate(Terminator::new(TerminatorKind::Return, span));
        block.unit()
    }
}

impl ToMir for TypedIf {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();

        let cond_rvalue = unpack!(block = self.cond.expr.to_mir(builder, block));
        let (block2, cond_op) = emit_temp(
            builder,
            block,
            cond_rvalue,
            &self.cond.expr.get_type(),
            span.clone(),
        );
        block = block2;

        let then_block = builder.start_block();
        let else_block = builder.start_block();
        let merge_block = builder.start_block();

        builder.switch_to(block);
        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond_op,
                targets: SwitchTargets::bool_branch(then_block, else_block),
            },
            span.clone(),
        ));

        let BlockAnd(then_exit, _) = self.then_branch.to_mir(builder, then_block);
        if !builder.is_block_terminated(then_exit) {
            builder.switch_to(then_exit);
            builder.terminate(Terminator::new(
                TerminatorKind::Goto(merge_block),
                span.clone(),
            ));
        }

        if let Some(else_b) = &self.else_branch {
            let BlockAnd(else_exit, _) = else_b.to_mir(builder, else_block);
            if !builder.is_block_terminated(else_exit) {
                builder.switch_to(else_exit);
                builder.terminate(Terminator::new(
                    TerminatorKind::Goto(merge_block),
                    span.clone(),
                ));
            }
        } else {
            builder.switch_to(else_block);
            builder.terminate(Terminator::new(
                TerminatorKind::Goto(merge_block),
                span.clone(),
            ));
        }

        merge_block.unit()
    }
}

impl ToMir for TypedWhile {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();

        let header_block = builder.start_block();
        let body_block = builder.start_block();
        let exit_block = builder.start_block();

        builder.switch_to(block);
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(header_block),
            span.clone(),
        ));

        builder.switch_to(header_block);
        let cond_rvalue = unpack!(block = self.cond.to_mir(builder, header_block));
        let (block2, cond_op) = emit_temp(
            builder,
            block,
            cond_rvalue,
            &self.cond.get_type(),
            span.clone(),
        );
        builder.switch_to(block2);
        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond_op,
                targets: SwitchTargets::bool_branch(body_block, exit_block),
            },
            span.clone(),
        ));

        builder.push_loop(header_block, exit_block);
        let BlockAnd(body_exit, _) = self.body.to_mir(builder, body_block);
        builder.pop_loop();
        if !builder.is_block_terminated(body_exit) {
            builder.switch_to(body_exit);
            builder.terminate(Terminator::new(
                TerminatorKind::Goto(header_block),
                span.clone(),
            ));
        }

        exit_block.unit()
    }
}

impl ToMir for TypedFor {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();

        unpack!(block = self.init.to_mir(builder, block));

        let header_block = builder.start_block();
        let body_block = builder.start_block();
        let update_block = builder.start_block();
        let exit_block = builder.start_block();

        builder.switch_to(block);
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(header_block),
            span.clone(),
        ));

        builder.switch_to(header_block);
        let cond_rvalue = unpack!(
            block = hades_ast::TypedExpr::Binary(self.cond.clone()).to_mir(builder, header_block)
        );
        let cond_ty = hades_ast::TypedExpr::Binary(self.cond.clone()).get_type();
        let (block2, cond_op) = emit_temp(builder, block, cond_rvalue, &cond_ty, span.clone());
        builder.switch_to(block2);
        builder.terminate(Terminator::new(
            TerminatorKind::SwitchInt {
                discriminant: cond_op,
                targets: SwitchTargets::bool_branch(body_block, exit_block),
            },
            span.clone(),
        ));

        builder.push_loop(update_block, exit_block);
        let BlockAnd(body_exit, _) = self.body.to_mir(builder, body_block);
        builder.pop_loop();
        if !builder.is_block_terminated(body_exit) {
            builder.switch_to(body_exit);
            builder.terminate(Terminator::new(
                TerminatorKind::Goto(update_block),
                span.clone(),
            ));
        }

        builder.switch_to(update_block);
        let update_expr = hades_ast::TypedExpr::Assign(self.update.clone());
        let BlockAnd(update_exit, _) = update_expr.to_mir(builder, update_block);
        if !builder.is_block_terminated(update_exit) {
            builder.switch_to(update_exit);
            builder.terminate(Terminator::new(
                TerminatorKind::Goto(header_block),
                span.clone(),
            ));
        }

        exit_block.unit()
    }
}

impl ToMir for TypedBreak {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        let loop_ctx = builder.current_loop().expect("break outside loop").clone();
        builder.switch_to(block);
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(loop_ctx.break_block),
            self.span.clone(),
        ));
        block.unit()
    }
}

impl ToMir for TypedContinue {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        let loop_ctx = builder
            .current_loop()
            .expect("continue outside loop")
            .clone();
        builder.switch_to(block);
        builder.terminate(Terminator::new(
            TerminatorKind::Goto(loop_ctx.continue_block),
            self.span.clone(),
        ));
        block.unit()
    }
}

impl ToMir for TypedDefer {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        let span = self.span.clone();
        let scratch = builder.start_block();
        let saved = builder.current_block();

        builder.switch_to(scratch);
        let BlockAnd(scratch_exit, _) = self.stmt.to_mir(builder, scratch);

        if !builder.is_block_terminated(scratch_exit) {
            builder.switch_to(scratch_exit);
            builder.terminate(Terminator::new(TerminatorKind::Unreachable, span.clone()));
        }

        let stmts = builder.drain_scratch_block(scratch);
        builder.push_defer(stmts, span);
        builder.switch_to(saved);
        block.unit()
    }
}
