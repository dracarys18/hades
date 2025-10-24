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

    pub fn check(&self, _: impl AsRef<std::path::Path>) {
        let source_trimmed = self.source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed, self.filename.to_string());
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

    pub fn compile(&self, path: impl AsRef<std::path::Path>) {
        let context = Context::create();

        let source_trimmed = self.source.trim();

        let mut lexer = lexer::Lexer::new(source_trimmed, self.filename.to_string());
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
        let prepared = analyzer
            .prepare(&program)
            .map_err(|err| {
                eprintln!("Error during semantic analysis: {err}");
                err
            })
            .expect("Semantic analysis preparation failed");

        prepared
            .analyze()
            .map_err(|err| {
                eprintln!("Error during semantic analysis: {err}");
                err
            })
            .expect("Semantic analysis failed");

        prepared
            .verify_module(&context, "main_module")
            .expect("Module verification failed");

        prepared
            .compile(&context, "main_module", path.as_ref())
            .map_err(|err| {
                eprintln!("Error during code generation: {err}");
                err
            })
            .expect("Code generation failed");
    }
}
