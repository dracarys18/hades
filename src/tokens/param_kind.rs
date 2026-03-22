use crate::error::Span;
use crate::tokens::{Ident, Selff};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ParamKind {
    Ident(Ident),
    Self_(Selff),
}

impl ParamKind {
    pub fn span(&self) -> &Span {
        match self {
            ParamKind::Ident(ident) => ident.span(),
            ParamKind::Self_(s) => s.span(),
        }
    }

    pub fn name(&self) -> Ident {
        match self {
            ParamKind::Ident(ident) => ident.clone(),
            ParamKind::Self_(selff) => Ident::new(String::from("self"), selff.span().clone()),
        }
    }
}
