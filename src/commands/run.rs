use anyhow::Result;
use clap::Args;
use elements::{LockTime, Sequence};
use simfony::dummy_env;
use simplicity::{
    ffi::tests::{run_program, TestUpTo},
    BitMachine,
};
use std::path::PathBuf;

use crate::{
    commands::{
        build::{compile_program, satisfy_program},
        BuildArgs,
    },
    helpers::{load_arguments, load_witness},
    tracker,
};

#[derive(Args, Debug)]
pub struct RunArgs {
    #[command(flatten)]
    pub build: BuildArgs,

    /// Path to file with arguments
    #[arg(long)]
    pub param: Option<PathBuf>,

    /// Print debug logs
    #[arg(long)]
    pub logging: Option<Logging>,

    /// Lock time
    /// See https://learnmeabitcoin.com/technical/transaction/locktime/
    #[arg(long)]
    pub lock_time: Option<u32>,

    /// Sequence
    /// See https://learnmeabitcoin.com/technical/transaction/input/sequence/
    #[arg(long)]
    pub sequence: Option<u32>,
}

#[derive(clap::ValueEnum, Clone, PartialEq, PartialOrd, Debug)]
pub enum Logging {
    #[clap(name = "info")]
    Info,
    #[clap(name = "debug")]
    Debug,
    #[clap(name = "trace")]
    Trace,
}

pub fn run(args: RunArgs) -> Result<()> {
    let witness = if let Some(witness_path) = args.build.witness {
        load_witness(Some(&witness_path))?
    } else {
        Default::default()
    };

    let arguments = if let Some(param_path) = args.param {
        load_arguments(Some(&param_path))?
    } else {
        Default::default()
    };

    let compiled = compile_program(
        &args.build.entrypoint,
        arguments,
        args.logging.is_some(),
        args.build.mcpp_inc_path,
    )?;
    let satisfied = satisfy_program(compiled, witness, args.build.prune)?;
    let node = satisfied.redeem();
    let env = dummy_env::dummy_with(
        LockTime::from_consensus(args.lock_time.unwrap_or(0)),
        Sequence::from_consensus(args.sequence.unwrap_or(0)),
        false,
    );

    if let Some(logging) = args.logging {
        let mut machine = BitMachine::for_program(node)?;
        let mut tracker = tracker::Tracker {
            debug_symbols: satisfied.debug_symbols(),
            debug_logs: logging >= Logging::Debug,
            jet_traces: logging == Logging::Trace,
        };
        let res = machine.exec_with_tracker(node, &env, &mut tracker)?;
        println!("Result: {}", res);
    } else {
        let (program_bytes, witness_bytes) = node.encode_to_vec();
        run_program(
            &program_bytes,
            &witness_bytes,
            TestUpTo::Everything,
            None,
            Some(env.c_tx_env()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run program: {}", e))?;
    }

    Ok(())
}
