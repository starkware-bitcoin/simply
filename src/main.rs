use clap::Parser;

mod commands;
mod tracker;

use commands::{handle_build, handle_debug, handle_run, Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build(args) => handle_build(args),
        Commands::Run(args) => handle_run(args),
        Commands::Debug(args) => handle_debug(args),
    };

    if let Err(err) = result {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
