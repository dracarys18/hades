use crate::{consts, lexer, parser};

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

    pub fn compile(&self, path: impl AsRef<std::path::Path>) {
        let source_trimmed = self.source.trim();

        // Lex the chars into tokens
        let mut lexer = lexer::Lexer::new(source_trimmed, self.filename.to_string());
        lexer
            .tokenize()
            .map_err(|err| eprintln!("{err}"))
            .expect("Tokenizing failed");

        // Parse the tokens into an AST
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

        program.compile_program(path);
    }
}
