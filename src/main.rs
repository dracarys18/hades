mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod semantic;
mod tokens;

use codegen::CodeGen;
use inkwell::context::Context;
use parser::Parser;

fn main() {
    let source = r#"
        fn main(): int {
            print("What the hell");
            print(42);
            print(3.14);
            return 0;
        }
    "#;

    println!("=== Parsing valid code ===");
    parse_and_report(source, "main.hd");

    let error_source = r#"
        fn broken_function(: int {
            let x =
            if y {
                return x
            }
        }
    "#;

    println!("\n=== Parsing invalid code ===");
    parse_and_report(error_source, "broken.hd");
}

fn parse_and_report(source: &str, filename: &str) {
    let source_trimmed = source.trim();

    let mut lexer = lexer::Lexer::new(source_trimmed, filename.to_string());
    match lexer.tokenize() {
        Ok(_) => {
            let mut parser = Parser::new(lexer.get_tokens().clone(), filename.to_string());

            match parser.parse() {
                Ok(program) => {
                    println!("{program:#?}");

                    // Generate LLVM IR
                    let context = Context::create();
                    let mut codegen = CodeGen::new(&context, "main_module");

                    match codegen.compile(program) {
                        Ok(_) => {
                            println!("Code generation successful!");

                            // Write IR to file
                            match codegen.write_ir_to_file("output.ll") {
                                Ok(_) => println!("LLVM IR written to: output.ll"),
                                Err(e) => {
                                    println!("Warning: Failed to write IR to file: {}", e)
                                }
                            }

                            println!("\n=== LLVM IR Output ===");
                            codegen.print_ir();
                        }
                        Err(e) => {
                            println!("Code generation failed: {}", e);
                        }
                    }
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
