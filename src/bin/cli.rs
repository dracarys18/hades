use clap::Parser;
use hades::{cmd, compiler, consts};
use inkwell::context::Context;
use std::{path::PathBuf, process::Command};

fn main() {
    let cmd = cmd::Cli::parse();

    match cmd.command {
        cmd::Commands::Build(args) => {
            let compiler = compiler::Compiler::new();
            compiler.prepare();

            let path = args.output.unwrap_or_else(|| {
                std::path::PathBuf::from(format!("{}/output", consts::BUILD_PATH))
            });

            compiler.compile(&args.source, path);
        }

        cmd::Commands::Check(args) => {
            let source = std::fs::read_to_string(&args.source).expect("Failed to read source file");
            let compiler = compiler::Compiler::new();

            compiler.prepare();
            compiler.check(&source, args.source.to_str().unwrap());
        }

        cmd::Commands::Run(args) => {
            let compiler = compiler::Compiler::new();
            compiler.prepare();

            let path = args
                .output
                .unwrap_or_else(|| PathBuf::from(format!("{}/output", consts::BUILD_PATH)));

            let res = compiler.compile(&args.source, &path);

            if res {
                Command::new(path)
                    .status()
                    .expect("Failed to run the compiled program");
            }
        }

        cmd::Commands::EmitLlvm(args) => {
            let compiler = compiler::Compiler::new();
            compiler.prepare();

            let context = Context::create();

            if let Err(e) = compiler.emit_llvm(&args.source, &context) {
                eprintln!("Failed to emit LLVM IR: {e}");
            }
        }

        cmd::Commands::PrintAst(args) => {
            let source = std::fs::read_to_string(&args.source).expect("Failed to read source file");
            let compiler = compiler::Compiler::new();
            compiler.prepare();
            compiler.print_ast(&source, args.source.to_str().unwrap());
        }
    }
}
