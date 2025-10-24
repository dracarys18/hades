use crate::ast::Types;
use crate::error::Span;
use crate::tokens::Ident;
use crate::typed_ast::function::FunctionSignature;
use indexmap::{IndexMap, indexmap};
use once_cell::sync::Lazy;

pub static BUILTIN_FUNCTIONS: Lazy<IndexMap<Ident, FunctionSignature>> = Lazy::new(|| {
    indexmap! {
        Ident::new(String::from("printf"), Span::default()) => FunctionSignature::new_variadic(Types::Int)
    }
});
