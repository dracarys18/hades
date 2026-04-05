use crate::ast::ArrayType;
use crate::ast::Types;
use crate::error::Span;
use crate::tokens::{Ident, ParamKind};
use crate::typed_ast::function::FunctionSignature;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

pub static BUILTIN_FUNCTIONS: Lazy<IndexMap<Ident, FunctionSignature>> = Lazy::new(|| {
    indexmap! {
        Ident::new(String::from("len"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("arr"), Span::default())) => Types::Generic(vec![
                    Types::Array(ArrayType::IntArray(0)),
                    Types::Array(ArrayType::FloatArray(0)),
                    Types::Array(ArrayType::StringArray(0)),
                    Types::Array(ArrayType::BoolArray(0)),
                    Types::Array(ArrayType::CharArray(0)),
                ]),
            },
            Types::Int,
            None,
        ),
    }
});
