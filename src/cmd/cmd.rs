use super::args::{BuildArgs, CheckArgs, EmitLlvmArgs, PrintAstArgs, RunArgs};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "hades")]
#[command(
    about = "A toy systems programming language",
    long_about = "Hades aims to be a fast systems programming language which gives you freedom to do whatever you want with enough checks to keep you safe."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build(BuildArgs),
    Check(CheckArgs),
    Run(RunArgs),
    EmitLlvm(EmitLlvmArgs),
    PrintAst(PrintAstArgs),
}
