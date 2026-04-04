use crate::ast::ArrayType;
use crate::ast::Types;
use crate::error::Span;
use crate::tokens::{Ident, ParamKind};
use crate::typed_ast::function::FunctionSignature;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

pub static BUILTIN_FUNCTIONS: Lazy<IndexMap<Ident, FunctionSignature>> = Lazy::new(|| {
    indexmap! {
        Ident::new(String::from("printf"), Span::default()) => FunctionSignature::new_variadic(Types::Int),
        Ident::new(String::from("sin"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("cos"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("sqrt"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("pow"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
                ParamKind::Ident(Ident::new(String::from("y"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("powi"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
                ParamKind::Ident(Ident::new(String::from("n"), Span::default())) => Types::Int,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("exp"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("exp2"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("log"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("log10"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("log2"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("floor"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("ceil"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("trunc"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
        Ident::new(String::from("round"), Span::default()) => FunctionSignature::new(
            indexmap! {
                ParamKind::Ident(Ident::new(String::from("x"), Span::default())) => Types::Float,
            },
            Types::Float,
            None,
        ),
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
