mod ast;
mod codegen;
mod compiler;
mod consts;
mod error;
mod evaluator;
mod lexer;
mod macros;
mod parser;
mod semantic;
mod tokens;
mod typed_ast;

fn main() {
    let source = r#"
        fn main(): int {
            printf("End of test\n");
            return 0;
        }
    "#;

    println!("=== Parsing valid code ===");
    parse_and_report(source, "main.hd");
}

fn parse_and_report(source: &str, filename: &str) {
    let compiler = compiler::Compiler::new(source, filename);
    compiler.prepare();

    let path = format!("{}/output", consts::BUILD_PATH);
    compiler.compile(path);
}

fn parse_and_check(source: &str, filename: &str) {
    let compiler = compiler::Compiler::new(source, filename);
    compiler.prepare();
    let path = format!("{}/output", consts::BUILD_PATH);
    compiler.check(path);
}
