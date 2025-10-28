mod ast;
mod cmd;
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

use clap::Parser;
use std::{path::PathBuf, process::Command};

fn main() {
    let cmd = cmd::Cli::parse();

    match cmd.command {
        cmd::Commands::Build(args) => {
            let source = std::fs::read_to_string(&args.source).expect("Failed to read source file");
            let compiler = compiler::Compiler::new(&source, args.source.to_str().unwrap());

            compiler.prepare();
            let path = args.output.unwrap_or_else(|| {
                std::path::PathBuf::from(format!("{}/output", consts::BUILD_PATH))
            });

            compiler.compile(path);
        }

        cmd::Commands::Check(args) => {
            let source = std::fs::read_to_string(&args.source).expect("Failed to read source file");
            let compiler = compiler::Compiler::new(&source, args.source.to_str().unwrap());

            compiler.prepare();
            compiler.check();
        }

        cmd::Commands::Run(args) => {
            let source = std::fs::read_to_string(&args.source).expect("Failed to read source file");
            let compiler = compiler::Compiler::new(&source, args.source.to_str().unwrap());
            compiler.prepare();

            let path = args
                .output
                .unwrap_or_else(|| PathBuf::from(format!("{}/output", consts::BUILD_PATH)));

            let res = compiler.compile(&path);

            if res {
                Command::new(path)
                    .status()
                    .expect("Failed to run the compiled program");
            }
        }
    }
}
