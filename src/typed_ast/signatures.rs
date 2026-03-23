use crate::module::ModulePath;
use crate::tokens::FunctionName;
use crate::typed_ast::function::FunctionSignature;
use crate::typed_ast::struc::Structs;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct ModuleSignatures {
    pub path: ModulePath,
    pub functions: IndexMap<FunctionName, FunctionSignature>,
    pub structs: Structs,
}
