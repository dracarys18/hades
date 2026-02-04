use crate::ast::Program;
use crate::module::Registry;
use crate::semantic::analyzer::{Analyzer, Unprepared};
use crate::{consts, lexer, parser};
use inkwell::context::Context;
use std::path::Path;

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
                    e.eprint(source_trimmed);
                }
                return;
            }
        };

        let analyzer = Analyzer::<Unprepared>::new();
        let analyzer = analyzer
            .prepare(&program)
            .map_err(|err| {
                eprintln!("Error during semantic analysis: {err}");
                err
            })
            .expect("Semantic analysis preparation failed");

        analyzer
            .analyze()
            .map_err(|err| {
                eprintln!("Error during semantic analysis: {err}");
                err
            })
            .expect("Semantic analysis failed");
    }

    pub fn compile(&self, entry_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> bool {
        let entry_path = entry_path.as_ref();
        let output_path = output_path.as_ref();
        let project_dir = if entry_path.is_dir() {
            entry_path
        } else {
            entry_path.parent().unwrap_or_else(|| Path::new("."))
        };

        let mut registry = Registry::new(project_dir);

        let program = match registry.load(entry_path) {
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
                eprintln!("Error during semantic analysis: {err}");
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
        let entry_path = entry_path.as_ref();
        let project_dir = if entry_path.is_dir() {
            entry_path
        } else {
            entry_path.parent().unwrap_or_else(|| Path::new("."))
        };

        let mut registry = Registry::new(project_dir);
        let program = registry.load(entry_path).map_err(|e| e.to_string())?;

        let analyzer = Analyzer::<Unprepared>::new();
        let prepared = analyzer.prepare(&program).map_err(|e| e.to_string())?;
        prepared.analyze().map_err(|e| e.to_string())?;

        let ir = prepared
            .generate_ir(context, consts::MAIN_MODULE_NAME)
            .map_err(|e| e.to_string())?;

        println!("{}", ir);
        Ok(())
    }

    pub fn print_ast(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();
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
                    e.eprint(source_trimmed);
                }
                return;
            }
        };
        println!("{:#?}", program);
    }
}
