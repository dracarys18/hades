use hades_ast::Types;

use crate::id::Id;

pub(crate) struct Local {
    id: Id,
    typ: Types,
}

impl Local {
    pub(crate) fn new(typ: Types) -> Self {
        Self { id: Id::new(), typ }
    }

    pub(crate) fn next(&mut self, typ: Types) -> Self {
        let id = self.id.next();
        let local = Self { id, typ };
        local
    }
}
