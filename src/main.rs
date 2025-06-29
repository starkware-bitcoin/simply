use clap::Parser;

mod commands;
mod helpers;
mod tracker;

use commands::{build, run, Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build(args) => build(args),
        Commands::Run(args) => run(args),
    };

    if let Err(err) = result {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
