use crate::ast::Types;
use crate::error::Span;
use crate::tokens::Ident;
use crate::typed_ast::function::FunctionSignature;
use indexmap::IndexMap;
use once_cell::sync::Lazy;

pub static BUILTIN_FUNCTIONS: Lazy<IndexMap<Ident, FunctionSignature>> = Lazy::new(|| {
    let mut map = IndexMap::new();

    map.insert(
        Ident::new(String::from("print"), Span::default()),
        FunctionSignature::new(
            vec![Types::Generic(vec![
                Types::String,
                Types::Int,
                Types::Float,
            ])],
            Types::Void,
        ),
    );

    map
});
