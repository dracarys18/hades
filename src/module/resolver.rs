use crate::ast::{Import, ImportPrefix};
use crate::module::error::ModuleError;
use crate::module::path::ModulePath;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Resolver {
    project_dir: PathBuf,
    stdlib_dir: PathBuf,
}

impl Resolver {
    pub fn new(project_dir: impl AsRef<Path>, stdlib_dir: impl AsRef<Path>) -> Self {
        Self {
            project_dir: project_dir.as_ref().to_path_buf(),
            stdlib_dir: stdlib_dir.as_ref().to_path_buf(),
        }
    }

    pub fn resolve(&self, import: &Import) -> Result<ModulePath, ModuleError> {
        match import.prefix {
            ImportPrefix::Std => Ok(ModulePath::Std(import.module.clone())),
            ImportPrefix::Local => Ok(ModulePath::Local(import.module.clone())),
        }
    }

    pub fn to_file_path(&self, module: &ModulePath) -> Result<PathBuf, ModuleError> {
        let mut path = match module {
            ModulePath::Std(_) => self.stdlib_dir.clone(),
            ModulePath::Local(_) => self.project_dir.clone(),
        };

        path.push(module.name());
        path.set_extension("hd");

        if !path.exists() {
            return Err(ModuleError::NotFound(module.to_string()));
        }

        Ok(path)
    }
}
