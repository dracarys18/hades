use crate::semantic::analyzer::{Analyzer, Unprepared};
use crate::{consts, lexer, parser};
use inkwell::context::Context;

pub struct Compiler<'a> {
    source: &'a str,
    filename: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, filename: &'a str) -> Self {
        Self { source, filename }
    }

    pub fn prepare(&self) {
        std::fs::create_dir_all(consts::BUILD_PATH).expect("Failed to create build directory");
    }

    pub fn check(&self) {
        let source_trimmed = self.source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), self.filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");

        let mut parser = parser::Parser::new(lexer.into_tokens(), self.filename.to_string());
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

    pub fn compile(&self, path: impl AsRef<std::path::Path>) -> bool {
        let context = Context::create();

        let source_trimmed = self.source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), self.filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");

        let mut parser = parser::Parser::new(lexer.into_tokens(), self.filename.to_string());
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(err) => {
                let err = err.into_errors();
                for e in err {
                    e.eprint(source_trimmed);
                }
                return false;
            }
        };

        let analyzer = Analyzer::<Unprepared>::new();

        let prepared = analyzer.prepare(&program);
        if let Err(err) = prepared {
            eprintln!("Error during semantic analysis: {err}");
            return false;
        }

        let prepared = prepared.unwrap();

        if let Err(err) = prepared.analyze() {
            eprintln!("Error during semantic analysis: {err}");
            return false;
        }

        if let Err(err) = prepared.verify_module(&context, consts::MAIN_MODULE_NAME) {
            eprintln!("Module verification failed: {err}");
            return false;
        }

        if let Err(err) = prepared.compile(&context, consts::MAIN_MODULE_NAME, path.as_ref()) {
            eprintln!("Module optimization failed: {err}");
            return false;
        }

        if let Err(err) = prepared.cleanup(path.as_ref()) {
            eprintln!("Module optimization failed: {err}");
            return false;
        }

        true
    }

    pub fn emit_llvm(
        &self,
        context: &inkwell::context::Context,
        _source_path: &std::path::Path,
    ) -> Result<(), String> {
        let source_trimmed = self.source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed.as_bytes(), self.filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| format!("{err}"))
            .expect("Tokenizing failed");

        let mut parser = parser::Parser::new(lexer.into_tokens(), self.filename.to_string());
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(err) => {
                let err = err.into_errors();
                for e in err {
                    e.eprint(source_trimmed);
                }
                return Err("Parsing failed".to_string());
            }
        };

        let analyzer = Analyzer::<Unprepared>::new();

        let prepared = analyzer.prepare(&program).map_err(|e| e.to_string())?;
        prepared.analyze().map_err(|e| e.to_string())?;

        let ir = prepared
            .generate_ir(context, consts::MAIN_MODULE_NAME)
            .map_err(|e| e.to_string())?;

        println!("{}", ir);
        Ok(())
    }
}
