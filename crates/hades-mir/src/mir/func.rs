use hades_ast::{TypedFuncDef, TypedReturn};
use hades_tokens::Ident;

use crate::{BasicBlock, BlockAnd, BlockAndExt, RETURN_LOCAL, ToMir, unpack};
use crate::mir::builder::MirBuilder;
use crate::mir::guard::Guard;
use crate::mir::place::Place;
use crate::mir::stmt::MirStmt;
use crate::mir::terminator::Terminator;

pub struct MirFunction {
    pub name: hades_tokens::Name,
    pub guard: Guard,
}

impl ToMir for TypedFuncDef {
    type Output = MirFunction;

    fn to_mir(&self, builder: &mut MirBuilder, _block: BasicBlock) -> BlockAnd<MirFunction> {
        builder.enter_guard();

        builder.build_local(
            Ident::new("__return".to_string(), self.span.clone()),
            self.signature.return_type.clone(),
        );

        for (param_kind, param_ty) in self.signature.params.clone() {
            builder.build_local(param_kind.name(), param_ty);
        }

        let mut block = builder.start_block();

        if let Some(body) = &self.body {
            unpack!(block = body.to_mir(builder, block));
        }

        builder.terminate(block, Terminator::Return);

        let guard = builder.exit_guard();
        let mir_fn = MirFunction { name: self.name.clone(), guard };
        block.and(mir_fn)
    }
}

impl ToMir for TypedReturn {
    type Output = ();

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<()> {
        if let Some(expr) = &self.expr {
            let mut block = block;
            let rvalue = unpack!(block = expr.expr.to_mir(builder, block));
            builder.push_stmt(block, MirStmt::Assign(
                Place::local(RETURN_LOCAL),
                rvalue,
            ));
        }
        builder.terminate(block, Terminator::Return);
        block.unit()
    }
}
