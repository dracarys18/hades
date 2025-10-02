mod ast;
mod codegen;
mod compiler;
mod consts;
mod error;
mod lexer;
mod parser;
mod semantic;
mod tokens;

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
    let compiler = compiler::Compiler::new(source, filename);
    compiler.prepare();

    compiler.compile();
}
