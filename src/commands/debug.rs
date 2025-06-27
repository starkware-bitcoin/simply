use anyhow::{Context, Result};
use clap::Args;
use simfony::{dummy_env, Arguments, CompiledProgram, WitnessValues};
use simplicity::BitMachine;
use std::fs;
use std::path::PathBuf;

use crate::tracker;

#[derive(Args)]
pub struct DebugArgs {
    /// Path to the source file
    pub path: PathBuf,

    /// Path to the witness file
    #[arg(long)]
    pub witness: Option<PathBuf>,

    /// Path to file with arguments
    #[arg(long)]
    pub param: Option<PathBuf>,
}

fn parse_witness(content: Option<&str>) -> Result<WitnessValues> {
    content.map_or(Ok(WitnessValues::default()), |s| {
        serde_json::from_str(s).with_context(|| "Failed to parse witness")
    })
}

fn parse_arguments(content: Option<&str>) -> Result<Arguments> {
    content.map_or(Ok(Arguments::default()), |s| {
        serde_json::from_str(s).with_context(|| "Failed to parse arguments")
    })
}

pub fn handle_debug(args: DebugArgs) -> Result<()> {
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("Failed to read source file: {}", args.path.display()))?;

    let param_content =
        if let Some(param_path) = args.param {
            Some(fs::read_to_string(&param_path).with_context(|| {
                format!("Failed to read parameter file: {}", param_path.display())
            })?)
        } else {
            None
        };

    let witness_content =
        if let Some(witness_path) = args.witness {
            Some(fs::read_to_string(&witness_path).with_context(|| {
                format!("Failed to read witness file: {}", witness_path.display())
            })?)
        } else {
            None
        };

    let arguments = parse_arguments(param_content.as_deref())?;
    let compiled = CompiledProgram::new(source, arguments, true).map_err(|e| anyhow::anyhow!(e))?;
    let witness = parse_witness(witness_content.as_deref())?;
    let satisfied = compiled.satisfy(witness).map_err(|e| anyhow::anyhow!(e))?;
    let node = satisfied.redeem();
    println!("Node bounds: {:?}", node.bounds());
    let mut machine = BitMachine::for_program(node)?;
    let env = dummy_env::dummy();
    let mut tracker = tracker::Tracker {
        debug_symbols: satisfied.debug_symbols(),
    };
    let res = machine.exec_with_tracker(node, &env, &mut tracker)?;

    println!("Result: {}", res);
    Ok(())
}
