use crate::{ast::Types, consts::GOOLAG_MESSAGE, tokens::Ident};
use indexmap::IndexMap;

pub type Field = IndexMap<Ident, Types>;

#[derive(Debug, Clone, PartialEq)]
pub struct Structs {
    inner: IndexMap<Ident, Field>,
}

impl Structs {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, name: Ident, fields: IndexMap<Ident, Types>) -> bool {
        self.inner.insert(name, fields).is_none()
    }

    pub fn fields(&self, name: &Ident) -> Option<&Field> {
        self.inner.get(name)
    }

    pub fn field_index(&self, name: &Ident, field_name: &Ident) -> usize {
        let field = self.inner.get(name).expect(GOOLAG_MESSAGE);
        field.get_index_of(field_name).expect(GOOLAG_MESSAGE)
    }
}
