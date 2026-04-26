use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use hades_ast::{TypedFieldKind, TypedStructDef};

impl Visit for TypedStructDef {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let opaque_struct = context.context().opaque_struct_type(self.name.inner());
        opaque_struct.set_body(
            &self
                .fields
                .iter()
                .filter_map(|(_, field)| match field {
                    TypedFieldKind::Func(_method) => {
                        // Method bodies are emitted by MirModule via MirFunction visits.
                        None
                    }
                    TypedFieldKind::Var(_) => {
                        let typ = field.get_type();
                        context
                            .type_converter()
                            .to_llvm_type(&typ, context.module())
                            .ok()
                    }
                })
                .collect::<Vec<_>>(),
            false,
        );
        Ok(())
    }
}
