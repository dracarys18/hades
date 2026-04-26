use hades_ast::Types;
use hades_error::Span;

/// Index into `MirFunction::locals`.
/// `_0` is the return slot, `_1.._N` are parameters, rest are temporaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

impl LocalId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for LocalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{}", self.0)
    }
}

/// A local variable slot in a MIR function.
/// No `name` field — names are debuginfo, not a MIR concern.
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub ty: Types,
    pub span: Span,
}

impl LocalDecl {
    pub fn new(ty: Types, span: Span) -> Self {
        Self { ty, span }
    }
}
