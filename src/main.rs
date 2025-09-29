mod ast;
mod error;
mod lexer;
mod parser;
mod semantic;
mod tokens;

use parser::Parser;

fn main() {
    let source = r#"
        struct Point {
            x: int,
            y: int,
            f: float,
        }

        fn add(a: int, b: int): int {
            let c = a - b + b - c;
            let x = "hello world";
            let float = 20.12313123;
            let f = 20;
            let p = Point { x: 10, y: 20, f: 3.14 };
            if x>=10 && x<10 || x==10 {
                print(x);
            } else {
                print("x is less than 10");
            }
            b = foo(a, b);
            b = 1;
            while b {
                f-=1;
                f+=2;
            }

            for i = 0; i < 10; i = i + 1 {
                print("Hello world", i);
            }
            return c;
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
