use crate::ast::types::Types;
use crate::tokens::Ident;
use indexmap::IndexMap;

pub struct IdentMap {
    inner: Vec<IndexMap<Ident, Types>>,
}

impl IdentMap {
    pub fn empty() -> Self {
        Self {
            inner: vec![IndexMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.inner.push(IndexMap::new());
    }
    pub fn exit_scope(&mut self) {
        self.inner.pop();
    }

    pub fn insert(&mut self, name: Ident, typ: Types) -> Result<(), String> {
        if let Some(current_scope) = self.inner.last_mut() {
            if current_scope.contains_key(&name) {
                return Err(format!(
                    "Variable '{}' already declared in this scope",
                    name
                ));
            }
            current_scope.insert(name, typ);
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    pub fn lookup(&self, name: &Ident) -> Option<&Types> {
        for scope in self.inner.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
}
