use hades_ast::Types;

use super::place::Place;

#[derive(Debug, Clone, PartialEq)]
pub enum MirConst {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Null(Types),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Copy(Place),
    Ref(Place),
    Const(MirConst),
}
