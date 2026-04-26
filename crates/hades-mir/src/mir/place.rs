use hades_ast::Types;
use hades_tokens::Ident;

use super::local::LocalId;

/// A projection element applied to a place.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaceElem {
    /// Dereference: `*place`
    Deref,
    /// Field access: `place.field`
    Field {
        name: Ident,
        index: usize,
        ty: Types,
    },
    /// Array index: `place[local]`
    Index(LocalId),
}

/// A location in memory: a local optionally projected through fields/deref/index.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<PlaceElem>,
}

impl Place {
    /// A bare local with no projection.
    pub fn local(local: LocalId) -> Self {
        Self {
            local,
            projection: vec![],
        }
    }

    pub fn with_deref(local: LocalId) -> Self {
        Self {
            local,
            projection: vec![PlaceElem::Deref],
        }
    }

    pub fn with_field(local: LocalId, name: Ident, index: usize, ty: Types) -> Self {
        Self {
            local,
            projection: vec![PlaceElem::Field { name, index, ty }],
        }
    }

    pub fn with_index(local: LocalId, index_local: LocalId) -> Self {
        Self {
            local,
            projection: vec![PlaceElem::Index(index_local)],
        }
    }
}
