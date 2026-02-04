use crate::ast::Program;
use crate::lexer::Lexer;
use crate::module::error::ModuleError;
use crate::module::path::ModulePath;
use crate::module::resolver::Resolver;
use crate::parser::Parser;
use std::path::PathBuf;

pub struct Module {
    pub path: ModulePath,
    pub ast: Program,
}

pub struct Loader {
    resolver: Resolver,
}

impl Loader {
    pub fn new(resolver: Resolver) -> Self {
        Self { resolver }
    }

    pub fn load(&self, module_path: &ModulePath) -> Result<Module, ModuleError> {
        let file_path = self.resolver.to_file_path(module_path)?;
        let source = std::fs::read_to_string(&file_path)?;

        let mut lexer = Lexer::new(source.as_bytes(), file_path.to_string_lossy().to_string());
        lexer.tokenize().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Lexer error".to_string(),
        })?;

        let mut parser = Parser::new(lexer.into_tokens(), file_path.to_string_lossy().to_string());
        let ast = parser.parse().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Parse error".to_string(),
        })?;

        Ok(Module {
            path: module_path.clone(),
            ast,
        })
    }

    pub fn load_from_file(&self, file_path: PathBuf) -> Result<Module, ModuleError> {
        let source = std::fs::read_to_string(&file_path)?;

        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModuleError::NotFound(file_path.display().to_string()))?;

        let module_path = ModulePath::Local(file_stem.to_string());

        let mut lexer = Lexer::new(source.as_bytes(), file_path.to_string_lossy().to_string());
        lexer.tokenize().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Lexer error".to_string(),
        })?;

        let mut parser = Parser::new(lexer.into_tokens(), file_path.to_string_lossy().to_string());
        let ast = parser.parse().map_err(|_| ModuleError::ParseError {
            module: module_path.to_string(),
            error: "Parse error".to_string(),
        })?;

        Ok(Module {
            path: module_path,
            ast,
        })
    }
}
