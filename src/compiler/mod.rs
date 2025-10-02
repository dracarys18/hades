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

    pub fn compile(&self) {
        let source_trimmed = self.source.trim();
        let mut lexer = lexer::Lexer::new(source_trimmed, self.filename.to_string());
        match lexer.tokenize() {
            Ok(_) => {
                let mut parser =
                    parser::Parser::new(lexer.get_tokens().clone(), self.filename.to_string());

                match parser.parse() {
                    Ok(program) => {
                        println!("{program:#?}");
                        let output_path = format!("{}/output.ll", consts::BUILD_PATH);
                        program.compile_to_llvm(output_path);
                    }
                    Err(errors) => {
                        println!("Parsing failed:");
                        for e in errors.into_errors() {
                            e.eprint(source_trimmed);
                        }
                    }
                }
            }
            Err(lex_error) => {
                println!("Lexing failed: {lex_error}");
            }
        }
    }
}
