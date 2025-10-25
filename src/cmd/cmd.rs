use super::args::{BuildArgs, CheckArgs, RunArgs};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "hades")]
#[command(about = "A toy systems programming language", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build(BuildArgs),
    Check(CheckArgs),
    Run(RunArgs),
}
