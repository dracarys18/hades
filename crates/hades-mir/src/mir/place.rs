use hades_ast::Types;
use hades_tokens::Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceElem {
    Deref,
    Field {
        name: Ident,
        index: usize,
        ty: Types,
    },
    Index(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub local: usize,
    pub projection: Vec<PlaceElem>,
}

impl Place {
    pub fn local(local: usize) -> Self {
        Self { local, projection: vec![] }
    }

    pub fn with_deref(local: usize) -> Self {
        Self { local, projection: vec![PlaceElem::Deref] }
    }

    pub fn with_field(local: usize, name: Ident, index: usize, ty: Types) -> Self {
        Self { local, projection: vec![PlaceElem::Field { name, index, ty }] }
    }

    pub fn with_index(local: usize, index_local: usize) -> Self {
        Self { local, projection: vec![PlaceElem::Index(index_local)] }
    }
}
