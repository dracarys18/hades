use hades_ast::Types;
use hades_tokens::Ident;

pub struct Local {
    pub name: Ident,
    pub typ: Types,
}

impl Local {
    pub fn new(name: Ident, typ: Types) -> Self {
        Self { name, typ }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
