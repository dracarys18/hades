use crate::typed_ast::TypedFieldKind;
use hades_common::consts::GOOLAG_MESSAGE;
use hades_tokens::{Ident, Name};
use indexmap::IndexMap;

pub type Field = IndexMap<Ident, TypedFieldKind>;

#[derive(Debug, Clone, PartialEq)]
pub struct Structs {
    inner: IndexMap<Name, Field>,
}

impl Default for Structs {
    fn default() -> Self {
        Self::new()
    }
}

impl Structs {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, name: Name, fields: IndexMap<Ident, TypedFieldKind>) -> bool {
        self.inner.insert(name, fields).is_none()
    }

    pub fn fields(&self, name: &Name) -> Option<&Field> {
        self.inner.get(name)
    }

    pub fn field_index(&self, name: &Name, field_name: &Ident) -> usize {
        let field = self.inner.get(name).expect(GOOLAG_MESSAGE);
        field
            .iter()
            .filter(|(_, kind)| matches!(kind, TypedFieldKind::Var(_)))
            .enumerate()
            .find(|(_, (k, _))| *k == field_name)
            .map(|(idx, _)| idx)
            .expect(GOOLAG_MESSAGE)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Name, &Field)> {
        self.inner.iter()
    }
}
