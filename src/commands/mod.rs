use clap::{Parser, Subcommand};

mod build;
mod run;

pub use build::{build, BuildArgs};
pub use run::{run, RunArgs};

#[derive(Parser)]
#[command(name = "simfony")]
#[command(about = "Simfony language CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build a Simfony program
    Build(BuildArgs),

    /// Run a Simfony program
    Run(RunArgs),
}
