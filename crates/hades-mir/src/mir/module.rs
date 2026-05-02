use hades_ast::ModulePath;

use super::func::MirFunction;

pub struct MirModule {
    pub path: ModulePath,
    pub functions: Vec<MirFunction>,
}
