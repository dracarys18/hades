use crate::ast::Program;
use crate::lexer::Lexer;
use crate::module::error::ModuleError;
use crate::module::path::ModulePath;
use crate::module::resolver::Resolver;
use crate::parser::Parser;
use crate::stdlib::Library;
use std::path::PathBuf;

pub struct Module {
    pub path: ModulePath,
    pub ast: Program,
}

pub struct Loader {
    resolver: Resolver,
    lib: Library,
}

impl Loader {
    pub fn new(resolver: Resolver) -> Self {
        Self {
            resolver,
            lib: Library::new(),
        }
    }

    pub fn load(&self, module_path: &ModulePath) -> Result<Module, ModuleError> {
        match module_path {
            ModulePath::Local(_) => self.load_local_module(module_path),
            ModulePath::Std(name) => self.load_std_module(name, module_path),
        }
    }

    fn load_local_module(&self, module_path: &ModulePath) -> Result<Module, ModuleError> {
        let file_path = self.resolver.to_file_path(module_path)?;
        let source = std::fs::read_to_string(&file_path)?;
        self.parse_source(
            &source,
            module_path,
            file_path.to_string_lossy().to_string(),
        )
    }

    fn load_std_module(
        &self,
        name: &String,
        module_path: &ModulePath,
    ) -> Result<Module, ModuleError> {
        if let Some(source) = self.lib.get_module(name) {
            return self.parse_source(source, module_path, format!("std::{}", name));
        } else {
            return Err(ModuleError::NotFound(format!(
                "Standard library module '{}' not found",
                name
            )));
        }
    }

    pub fn load_from_file(&self, file_path: PathBuf) -> Result<Module, ModuleError> {
        let source = std::fs::read_to_string(&file_path)?;

        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModuleError::NotFound(file_path.display().to_string()))?;

        let module_path = ModulePath::Local(file_stem.to_string());
        self.parse_source(
            &source,
            &module_path,
            file_path.to_string_lossy().to_string(),
        )
    }

    fn parse_source(
        &self,
        source: &str,
        module_path: &ModulePath,
        filename: String,
    ) -> Result<Module, ModuleError> {
        let mut lexer = Lexer::new(source.as_bytes(), filename.clone());
        lexer.tokenize().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Lexer error".to_string(),
        })?;

        let mut parser = Parser::new(lexer.into_tokens(), filename);
        let ast = parser.parse().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Parse error".to_string(),
        })?;

        Ok(Module {
            path: module_path.clone(),
            ast,
        })
    }
}
