use hades_ast::Types;
use hades_tokens::Ident;

#[derive(Clone)]
pub enum ConstVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Null(Types),
}

#[derive(Clone)]
pub enum Operand {
    Copy(Place),
    Constant(ConstVal),
}

#[derive(Clone)]
pub enum PlaceElem {
    Field(Ident),
    Index(Operand),
    Deref,
}

#[derive(Clone)]
pub struct Place {
    pub local: usize,
    pub projection: Vec<PlaceElem>,
}

impl Place {
    pub fn local(idx: usize) -> Self {
        Place { local: idx, projection: vec![] }
    }
}
