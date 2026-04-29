use hades_ast::Types;
use hades_tokens::Ident;

pub(crate) struct Local {
    pub name: Ident,
    pub typ: Types,
}

impl Local {
    pub(crate) fn new(name: Ident, typ: Types) -> Self {
        Self { name, typ }
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }
}
