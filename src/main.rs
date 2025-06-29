use clap::Parser;

mod commands;
mod esplora;
mod helpers;
mod rpc;
mod script;
mod tracker;
mod transaction;

use commands::{build, deposit, run, test, withdraw, Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build(args) => build(args),
        Commands::Run(args) => run(args),
        Commands::Test(args) => test(args),
        Commands::Deposit(args) => deposit(args),
        Commands::Withdraw(args) => withdraw(args),
    };

    if let Err(err) = result {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
