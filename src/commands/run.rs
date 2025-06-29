use anyhow::Result;
use clap::Args;
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

#[derive(Args)]
pub struct RunArgs {
    #[command(flatten)]
    pub build: BuildArgs,

    /// Path to file with arguments
    #[arg(long)]
    pub param: Option<PathBuf>,

    /// Print debug logs
    #[arg(long)]
    pub logging: Option<Logging>,
}

#[derive(clap::ValueEnum, Clone, PartialEq, PartialOrd)]
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

    let compiled = compile_program(&args.build.path, arguments, args.logging.is_some())?;
    let satisfied = satisfy_program(compiled, witness, args.build.prune)?;
    let node = satisfied.redeem();
    let env = dummy_env::dummy();

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
