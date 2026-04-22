use hades_error::Span;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Selff {
    span: Span,
}

impl Selff {
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
