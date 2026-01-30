use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(required = true)]
    pub source: PathBuf,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(required = true)]
    pub source: PathBuf,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(required = true)]
    pub source: PathBuf,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct EmitLlvmArgs {
    #[arg(required = true)]
    pub source: PathBuf,
}

#[derive(Debug, Args)]
pub struct PrintAstArgs {
    #[arg(required = true)]
    pub source: PathBuf,
}
