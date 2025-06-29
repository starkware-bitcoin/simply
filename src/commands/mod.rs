use clap::{Parser, Subcommand};

mod build;
mod deposit;
mod run;
mod test;
mod withdraw;

pub use build::{build, BuildArgs};
pub use deposit::{deposit, DepositArgs};
pub use run::{run, Logging, RunArgs};
pub use test::{test, TestArgs};
pub use withdraw::{withdraw, WithdrawArgs};

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

    /// Run tests
    Test(TestArgs),

    /// Generate a P2TR address to make a deposit
    Deposit(DepositArgs),

    /// Spend a transaction output
    Withdraw(WithdrawArgs),
}
