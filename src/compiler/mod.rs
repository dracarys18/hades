use crate::ast::Program;
use crate::module::Registry;
use crate::semantic::analyzer::{Analyzer, Unprepared};
use crate::{consts, lexer, parser};
use ariadne::{Cache, Source};
use inkwell::context::Context;
use std::collections::HashMap;
use std::path::Path;

/// A simple cache that loads files from disk on-demand
struct FileSourceCache {
    sources: HashMap<String, Source<String>>,
}

impl FileSourceCache {
    fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
}

impl Cache<String> for FileSourceCache {
    type Storage = String;

    fn fetch(&mut self, id: &String) -> Result<&Source<String>, Box<dyn std::fmt::Debug + '_>> {
        if !self.sources.contains_key(id) {
            // Try to load the file from disk
            let content =
                std::fs::read_to_string(id).map_err(|e| Box::new(e) as Box<dyn std::fmt::Debug>)?;
            self.sources.insert(id.clone(), Source::from(content));
        }
        Ok(self.sources.get(id).unwrap())
    }

    fn display<'a>(&self, id: &'a String) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(id.as_str()))
    }
}

pub struct Compiler {}

impl<'a> Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn prepare(&self) {
        std::fs::create_dir_all(consts::BUILD_PATH).expect("Failed to create build directory");
    }

    pub fn check(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();

        // Create a simple cache for the single source file using sources()
        let mut cache = ariadne::sources(vec![(filename.to_string(), source_trimmed)]);

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");

        let mut parser = parser::Parser::new(lexer.into_tokens(), filename.to_string());
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(err) => {
                let err = err.into_errors();
                for e in err {
                    e.eprint(&mut cache);
                }
                return;
            }
        };

        let analyzer = Analyzer::<Unprepared>::new();
        let analyzer = match analyzer.prepare(&program) {
            Ok(a) => a,
            Err(err) => {
                err.into_error().eprint(&mut cache);
                return;
            }
        };

        if let Err(err) = analyzer.analyze() {
            eprintln!("Error during semantic analysis: {err}");
            return;
        }
    }

    pub fn compile(&self, entry_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> bool {
        let output_path = output_path.as_ref();

        // Use custom file cache that loads from disk on-demand
        let mut cache = FileSourceCache::new();

        let program = match Registry::load(entry_path) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Failed to load modules: {err}");
                return false;
            }
        };

        let context = Context::create();
        let analyzer = Analyzer::<Unprepared>::new();

        let prepared = match analyzer.prepare(&program) {
            Ok(p) => p,
            Err(err) => {
                err.into_error().eprint(&mut cache);
                return false;
            }
        };

        if let Err(err) = prepared.analyze() {
            eprintln!("Error during semantic analysis: {err}");
            return false;
        }

        if let Err(err) = prepared.verify_module(&context, consts::MAIN_MODULE_NAME) {
            eprintln!("Module verification failed: {err}");
            return false;
        }

        if let Err(err) = prepared.compile(&context, consts::MAIN_MODULE_NAME, output_path) {
            eprintln!("Compilation failed: {err}");
            return false;
        }

        if let Err(err) = prepared.cleanup(output_path) {
            eprintln!("Cleanup failed: {err}");
            return false;
        }

        true
    }

    pub fn emit_llvm(
        &self,
        entry_path: impl AsRef<Path>,
        context: &inkwell::context::Context,
    ) -> Result<(), String> {
        let program = Registry::load(entry_path).map_err(|e| e.to_string())?;

        let analyzer = Analyzer::<Unprepared>::new();
        let prepared = analyzer
            .prepare(&program)
            .map_err(|e| e.into_error().to_string())?;
        prepared.analyze().map_err(|e| e.to_string())?;

        let ir = prepared
            .generate_ir(context, consts::MAIN_MODULE_NAME)
            .map_err(|e| e.to_string())?;

        println!("{}", ir);
        Ok(())
    }

    pub fn print_ast(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();
        let mut cache = ariadne::sources(vec![(filename.to_string(), source_trimmed)]);

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");
        let mut parser = parser::Parser::new(lexer.into_tokens(), filename.to_string());
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(err) => {
                let err = err.into_errors();
                for e in err {
                    e.eprint(&mut cache);
                }
                return;
            }
        };
        println!("{:#?}", program);
    }
}
