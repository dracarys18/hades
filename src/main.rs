mod ast;
mod codegen;
mod compiler;
mod consts;
mod error;
mod evaluator;
mod lexer;
mod parser;
mod semantic;
mod tokens;

fn main() {
    let source = r#"
        fn main(): int {
            print("Hello, world!");
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

    let path = format!("{}/output", consts::BUILD_PATH);
    compiler.compile(path);
}
