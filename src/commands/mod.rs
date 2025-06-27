use clap::{Parser, Subcommand};

mod build;
mod debug;
mod run;

pub use build::{handle_build, BuildArgs};
pub use debug::{handle_debug, DebugArgs};
pub use run::{handle_run, RunArgs};

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

    /// Debug a Simfony program
    Debug(DebugArgs),
}
