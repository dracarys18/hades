use crate::ast::Types;
use crate::semantic::scope::Scope;

use crate::tokens::Ident;

#[derive(Debug, Clone, PartialEq)]
pub struct IdentMap {
    inner: Scope<Types>,
}

impl IdentMap {
    pub fn empty() -> Self {
        Self {
            inner: Scope::global(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.inner.enter_scope();
    }
    pub fn exit_scope(&mut self) {
        self.inner.exit_scope();
    }

    pub fn insert(&mut self, name: Ident, typ: Types) {
        self.inner.on_scope_mut(|node| {
            node.insert(name, typ);
        });
    }

    pub fn lookup<'a>(&'a self, name: &'a Ident) -> Option<&'a Types> {
        self.inner.lookup_scope(name)
    }
}
