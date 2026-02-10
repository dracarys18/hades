use crate::ast::Program;
use crate::module::Registry;
use crate::semantic::analyzer::{Analyzer, Unprepared};
use crate::{consts, lexer, parser};
use ariadne::{Cache, Source};
use inkwell::context::Context;
use std::path::PathBuf;
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt, fs,
    path::Path,
};

struct FileSourceCache {
    sources: HashMap<PathBuf, Source<String>>,
}

impl FileSourceCache {
    fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
}

impl Cache<PathBuf> for FileSourceCache {
    type Storage = String;

    fn fetch(&mut self, path: &PathBuf) -> Result<&Source, impl fmt::Debug> {
        Ok::<_, std::io::Error>(match self.sources.entry(path.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Source::from(fs::read_to_string(path)?)),
        })
    }
    fn display<'a>(&self, id: &'a PathBuf) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(id.display()))
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

    fn parse_single_file(
        source: &str,
        filename: &str,
        cache: impl Cache<PathBuf>,
    ) -> Option<Program> {
        let source_trimmed = source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");

        let mut parser = parser::Parser::new(lexer.into_tokens(), filename.to_string());
        match parser.parse() {
            Ok(prog) => Some(prog),
            Err(err) => {
                let err = err.into_errors();
                for e in err {
                    e.eprint(cache);
                }
                None
            }
        }
    }

    fn load_program(
        entry_path: impl AsRef<Path>,
        cache: &mut impl Cache<String>,
    ) -> Option<Program> {
        match Registry::load(entry_path) {
            Ok(p) => Some(p),
            Err(err) => {
                eprintln!("Failed to load modules: {err}");
                None
            }
        }
    }

    fn analyze_program(
        program: &Program,
        cache: &mut impl Cache<String>,
    ) -> Option<Analyzer<crate::semantic::analyzer::Prepared>> {
        let analyzer = Analyzer::<Unprepared>::new();
        let analyzer = match analyzer.prepare(program) {
            Ok(a) => a,
            Err(err) => {
                err.into_error().eprint(cache);
                return None;
            }
        };

        if let Err(err) = analyzer.analyze() {
            eprintln!("Error during semantic analysis: {err}");
            return None;
        }

        Some(analyzer)
    }

    pub fn check(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();
        let mut cache = ariadne::sources(vec![(PathBuf::from(filename), source_trimmed)]);

        let program = match Self::parse_single_file(source, filename, cache) {
            Some(p) => p,
            None => return,
        };

        Self::analyze_program(&program, cache);
    }

    pub fn compile(&self, entry_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> bool {
        let output_path = output_path.as_ref();
        let mut cache = FileSourceCache::new();

        let program = match Self::load_program(entry_path, &mut cache) {
            Some(p) => p,
            None => return false,
        };

        let analyzer = match Self::analyze_program(&program, &mut cache) {
            Some(a) => a,
            None => return false,
        };

        let context = Context::create();

        if let Err(err) = analyzer.verify_module(&context, consts::MAIN_MODULE_NAME) {
            eprintln!("Module verification failed: {err}");
            return false;
        }

        if let Err(err) = analyzer.compile(&context, consts::MAIN_MODULE_NAME, output_path) {
            eprintln!("Compilation failed: {err}");
            return false;
        }

        if let Err(err) = analyzer.cleanup(output_path) {
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
        let mut cache = FileSourceCache::new();

        let program = Self::load_program(entry_path, &mut cache)
            .ok_or_else(|| "Failed to load program".to_string())?;

        let analyzer = Self::analyze_program(&program, &mut cache)
            .ok_or_else(|| "Semantic analysis failed".to_string())?;

        let ir = analyzer
            .generate_ir(context, consts::MAIN_MODULE_NAME)
            .map_err(|e| e.to_string())?;

        println!("{}", ir);
        Ok(())
    }

    pub fn print_ast(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();
        let mut cache = ariadne::sources(vec![(filename.to_string(), source_trimmed)]);

        let program = match Self::parse_single_file(source, filename, &mut cache) {
            Some(p) => p,
            None => return,
        };

        println!("{:#?}", program);
    }
}
