use crate::{error::Span, impl_span};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Selff {
    span: Span,
}

impl Selff {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

impl_span!(Selff);
