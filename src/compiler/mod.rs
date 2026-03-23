use crate::module::{Module, ModulePath, Registry};
use crate::semantic::analyzer::{Analyzer, Unprepared};
use crate::{codegen::llvm::visit::program as codegen, consts, lexer, parser};
use ariadne::{Cache, Source};
use inkwell::context::Context;
use std::path::PathBuf;
use std::{
    collections::{hash_map::Entry, HashMap},
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

struct FileCache {
    sources: HashMap<PathBuf, Source<String>>,
}

impl FileCache {
    fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
}

impl From<Vec<(PathBuf, &str)>> for FileCache {
    fn from(files: Vec<(PathBuf, &str)>) -> Self {
        let mut cache = Self::new();
        for (path, source) in files {
            cache.sources.insert(path, Source::from(source.to_string()));
        }
        cache
    }
}

impl Cache<Path> for FileSourceCache {
    type Storage = String;

    fn fetch(&mut self, path: &Path) -> Result<&Source, impl fmt::Debug> {
        Ok::<_, std::io::Error>(match self.sources.entry(path.to_path_buf()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Source::from(fs::read_to_string(path)?)),
        })
    }
    fn display<'a>(&self, path: &'a Path) -> Option<impl fmt::Display + 'a> {
        Some(path.display())
    }
}

impl Cache<&Path> for FileSourceCache {
    type Storage = String;

    fn fetch(&mut self, path: &&Path) -> Result<&Source, impl fmt::Debug> {
        Cache::<Path>::fetch(self, *path)
    }
    fn display<'a>(&self, path: &'a &Path) -> Option<impl fmt::Display + 'a> {
        Cache::<Path>::display(self, *path)
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
    fn display<'a>(&self, path: &'a PathBuf) -> Option<impl fmt::Display + 'a> {
        Some(path.display())
    }
}

impl Cache<PathBuf> for FileCache {
    type Storage = String;

    fn fetch(&mut self, path: &PathBuf) -> Result<&Source, impl fmt::Debug> {
        self.sources
            .get(path)
            .ok_or_else(|| format!("Source not found: {}", path.display()))
    }
    fn display<'a>(&self, path: &'a PathBuf) -> Option<impl fmt::Display + 'a> {
        Some(path.display())
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

    pub fn check(&self, source: &'a str, filename: &'a str) -> bool {
        let source_trimmed = source.trim();
        let mut cache = FileCache::from(vec![(PathBuf::from(filename), source_trimmed)]);

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
                return false;
            }
        };

        let module = Module {
            path: ModulePath::Local(filename.to_string()),
            ast: program,
            imports: vec![],
        };

        let analyzer = Analyzer::<Unprepared>::new();
        let analyzer = match analyzer.prepare(vec![module]) {
            Ok(a) => a,
            Err(err) => {
                err.into_error().eprint(&mut cache);
                return false;
            }
        };

        if let Err(err) = analyzer.analyze() {
            eprintln!("Error during semantic analysis: {err}");
            return false;
        }

        true
    }

    pub fn compile(&self, entry_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> bool {
        let output_path = output_path.as_ref();
        let mut cache = FileSourceCache::new();

        let modules = match Registry::load(entry_path) {
            Ok(m) => m,
            Err(err) => {
                eprintln!("Failed to load modules: {err}");
                return false;
            }
        };

        let llvm_ctx = Context::create();
        let analyzer = Analyzer::<Unprepared>::new();

        let prepared = match analyzer.prepare(modules) {
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

        let typed_modules = prepared.modules();
        let sig_map: HashMap<&ModulePath, _> = typed_modules
            .iter()
            .map(|m| (&m.path, &m.signatures))
            .collect();

        let mut obj_paths: Vec<PathBuf> = Vec::new();

        for typed_module in typed_modules {
            let obj_path =
                PathBuf::from(consts::BUILD_PATH).join(format!("{}.o", typed_module.path.name()));

            let import_sigs: Vec<_> = typed_module
                .imports
                .iter()
                .filter_map(|p| sig_map.get(p).copied())
                .collect();

            if let Err(err) = codegen::compile(typed_module, &import_sigs, &llvm_ctx, &obj_path) {
                eprintln!("Compilation failed for {}: {err}", typed_module.path);
                return false;
            }

            obj_paths.push(obj_path);
        }

        let mut cmd = std::process::Command::new("clang");
        for obj in &obj_paths {
            cmd.arg(obj);
        }
        cmd.arg("-o").arg(output_path).arg("-lc");

        let status = match cmd.status() {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Failed to invoke clang: {err}");
                return false;
            }
        };

        if !status.success() {
            eprintln!("Clang failed to create executable");
            return false;
        }

        for obj in &obj_paths {
            let _ = std::fs::remove_file(obj);
        }

        true
    }

    pub fn emit_llvm(
        &self,
        entry_path: impl AsRef<Path>,
        context: &inkwell::context::Context,
    ) -> Result<(), String> {
        let modules = Registry::load(entry_path).map_err(|e| e.to_string())?;

        let analyzer = Analyzer::<Unprepared>::new();
        let prepared = analyzer
            .prepare(modules)
            .map_err(|e| e.into_error().to_string())?;
        prepared.analyze().map_err(|e| e.to_string())?;

        let typed_modules = prepared.modules();
        let sig_map: HashMap<&ModulePath, _> = typed_modules
            .iter()
            .map(|m| (&m.path, &m.signatures))
            .collect();

        for typed_module in typed_modules {
            let import_sigs: Vec<_> = typed_module
                .imports
                .iter()
                .filter_map(|p| sig_map.get(p).copied())
                .collect();

            let ir =
                codegen::emit_ir(typed_module, &import_sigs, context).map_err(|e| e.to_string())?;

            println!("; === module: {} ===", typed_module.path);
            println!("{}", ir);
        }

        Ok(())
    }

    pub fn print_ast(&self, source: &'a str, filename: &'a str) {
        let source_trimmed = source.trim();
        let mut cache = FileCache::from(vec![(PathBuf::from(filename), source_trimmed)]);

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
