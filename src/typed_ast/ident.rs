use crate::ast::Types;
use crate::semantic::scope::Scope;

use crate::tokens::Ident;
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct IdentMap {
    inner: Scope<IndexMap<Ident, Types>>,
}

impl IdentMap {
    pub fn empty() -> Self {
        Self {
            inner: Scope::empty(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.inner.enter_scope(IndexMap::new());
    }
    pub fn exit_scope(&mut self) {
        self.inner.exit_scope();
    }

    pub fn current_scope(&self) -> Option<&IndexMap<Ident, Types>> {
        self.inner.current_scope()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut IndexMap<Ident, Types>> {
        self.inner.current_scope_mut()
    }

    pub fn insert(&mut self, name: Ident, typ: Types) -> Result<(), String> {
        if let Some(current_scope) = self.inner.current_scope_mut() {
            if current_scope.contains_key(&name) {
                return Err(format!("Variable '{name}' already declared in this scope",));
            }
            current_scope.insert(name, typ);
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    pub fn lookup<'a>(&'a self, name: &'a Ident) -> Option<&'a Types> {
        let get = |scope: &'a IndexMap<Ident, Types>| scope.get(name);
        let cond = |scope: &&'a IndexMap<Ident, Types>| scope.contains_key(name);

        self.inner.lookup_scope(cond, get)
    }
}

impl<'a> IntoIterator for &'a IdentMap {
    type Item = &'a IndexMap<Ident, Types>;
    type IntoIter = std::slice::Iter<'a, IndexMap<Ident, Types>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
