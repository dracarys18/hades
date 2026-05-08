use hades_ast::{FuncKind, FunctionSignature, TypedFuncDef, TypedReceiver, Types};
use hades_error::Span;
use hades_tokens::{Ident, Name, ParamKind};

use crate::mir::builder::MirBuilder;
use crate::mir::guard::Guard;
use crate::mir::terminator::{Terminator, TerminatorKind};
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

pub struct MirFunction {
    pub name: Name,
    pub receiver: Option<TypedReceiver>,
    pub return_type: Types,
    pub arg_count: usize,
    pub guard: Guard,
    pub signature: FunctionSignature,
    pub span: Span,
}

impl ToMir for TypedFuncDef {
    type Output = Option<MirFunction>;

    fn to_mir(
        &self,
        builder: &mut MirBuilder<'_>,
        _block: BasicBlock,
    ) -> BlockAnd<Option<MirFunction>> {
        match self.signature.kind {
            FuncKind::Extern { .. } | FuncKind::Intrinsic(_) => {
                return BasicBlock(0).and(None);
            }
            FuncKind::Normal => {}
        }

        let body = match &self.body {
            Some(b) => b,
            None => return BasicBlock(0).and(None),
        };

        let span = self.span.clone();
        builder.enter_guard();

        builder.build_local(
            Ident::new("_0".to_string(), span.clone()),
            self.signature.return_type.clone(),
        );

        let mut arg_count = 0usize;
        for (param_kind, param_ty) in self.signature.params() {
            match &param_kind {
                ParamKind::Self_(_) => {
                    builder.build_local(Ident::new("self".to_string(), span.clone()), param_ty);
                    arg_count += 1;
                }
                ParamKind::Ident(ident) => {
                    builder.build_local(ident.clone(), param_ty);
                    arg_count += 1;
                }
            }
        }

        let bb0 = builder.start_block();
        builder.switch_to(bb0);

        let mut block = bb0;
        unpack!(block = body.to_mir(builder, block));

        if !builder.is_block_terminated(block) {
            let deferred = builder.deferred_stmts();
            for stmt in deferred {
                builder.push_stmt(block, stmt);
            }
            builder.switch_to(block);
            builder.terminate(Terminator::new(TerminatorKind::Return, span.clone()));
        }

        let guard = builder.exit_guard();
        let mir_fn = MirFunction {
            name: self.name.clone(),
            receiver: self.signature.receiver.clone(),
            return_type: self.signature.return_type.clone(),
            arg_count,
            guard,
            signature: self.signature.clone(),
            span,
        };
        BasicBlock(0).and(Some(mir_fn))
    }
}
