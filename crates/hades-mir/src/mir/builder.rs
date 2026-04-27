use hades_ast::CompilerContext;

pub(crate) struct MirBuilder<'ctx> {
    ctx: &'ctx CompilerContext,
}

impl<'ctx> MirBuilder<'ctx> {
    pub fn new(_return_type: hades_ast::Types, _span: hades_error::Span) -> Self {
        Self {}
    }
}
