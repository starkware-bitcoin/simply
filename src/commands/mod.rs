use clap::{Parser, Subcommand};

mod build;
mod deposit;
mod run;
mod sign;
mod test;
mod withdraw;

pub use build::{build, BuildArgs};
pub use deposit::{deposit, DepositArgs};
pub use run::{run, Logging, RunArgs};
pub use sign::{sign, SignArgs};
pub use test::{test, TestArgs};
pub use withdraw::{withdraw, WithdrawArgs};

#[derive(Parser)]
#[command(name = "simply")]
#[command(about = "SimplicityHL language CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build a SimplicityHL program
    Build(BuildArgs),

    /// Run a SimplicityHL program
    Run(RunArgs),

    /// Run tests
    Test(TestArgs),

    /// Generate a P2TR address to make a deposit
    Deposit(DepositArgs),

    /// Spend a transaction output
    Withdraw(WithdrawArgs),

    /// Sign a message
    Sign(SignArgs),
}
