use super::{builder::MirBuilder, local::Local};
use hades_ast::TypedFuncDef;
use hades_tokens::Name;

use crate::{ToMir, mir::guard::Guard};

pub struct MirFunction {
    name: Name,
    guard: Guard,
}

impl ToMir for TypedFuncDef {
    type Output = MirFunction;

    fn to_mir(&self, builder: &mut MirBuilder) -> Self::Output {
        builder.enter_guard();
        builder.build_local(self.signature.return_type.clone());

        for param in self.signature.params.clone() {
            builder.build_local(param.1.clone());
        }

        self.body.as_ref().map(|b| b.to_mir(builder));

        MirFunction {
            name: self.name.clone(),
            guard: builder.exit_guard(),
        }
    }
}
