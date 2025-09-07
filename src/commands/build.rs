use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use simfony::{dummy_env, Arguments, CompiledProgram, SatisfiedProgram, WitnessValues};
use simplicity::human_encoding::Forest;
use simplicity::jet::Elements;
use simplicity::{BitIter, CommitNode};
use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};

use crate::helpers::{get_program_name, load_witness};

#[derive(Args, Clone, Debug)]
pub struct BuildArgs {
    /// Path to the source file
    /// Default: `./src/main.simf`
    #[arg(long, default_value = "./src/main.simf")]
    pub entrypoint: PathBuf,

    /// Path to the mcpp include directory
    /// If not provided, the program will be compiled without mcpp.
    #[arg(long)]
    pub mcpp_inc_path: Option<PathBuf>,

    /// Path to the witness file
    #[arg(long)]
    pub witness: Option<PathBuf>,

    /// Prune the program using the provided witness
    #[arg(long)]
    pub prune: bool,

    /// Write Simplicity assembly to a file
    #[arg(long)]
    pub assembly: bool,

    /// Output directory for the compiled program (will use `target` by default)
    #[arg(long, name = "target-dir", default_value = "./target")]
    pub target_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifacts {
    pub program: Vec<u8>,
    pub witness: Option<Vec<u8>>,
}

fn format_node_bounds(bounds: &simplicity::NodeBounds) -> String {
    format!(
        "Node bounds:\n  Extra cells: {}\n  Extra frames: {}\n  CPU cost: {}",
        bounds.extra_cells, bounds.extra_frames, bounds.cost
    )
}

fn write_build_output(
    target_dir: &PathBuf,
    program_name: &str,
    artifacts: BuildArtifacts,
    assembly: bool,
) -> Result<()> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(target_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            target_dir.display()
        )
    })?;

    // Create output file path
    // TODO(m_kus): add witness hash to distinct between programs pruned on different witnesses.
    let output_file = target_dir.join(program_name).with_extension("json");

    // Serialize artifacts to JSON and write to file
    let json_content = serde_json::to_string_pretty(&artifacts)
        .with_context(|| "Failed to serialize build artifacts to JSON")?;

    fs::write(&output_file, json_content)
        .with_context(|| format!("Failed to write output file: {}", output_file.display()))?;

    if assembly {
        let iter = BitIter::from(artifacts.program.into_iter());
        let commit = CommitNode::decode(iter).with_context(|| "failed to decode program")?;
        let prog = Forest::<Elements>::from_program(commit);

        let assembly_file = target_dir.join(program_name).with_extension("simp");

        fs::write(&assembly_file, prog.string_serialize()).with_context(|| {
            format!("Failed to write assembly file: {}", assembly_file.display())
        })?;

        println!("Assembly written to: {}", assembly_file.display());
    }

    println!("Build artifacts written to: {}", output_file.display());
    Ok(())
}

pub fn compile_program(
    source_path: &Path,
    arguments: Arguments,
    debug_symbols: bool,
    mcpp_inc_path: Option<PathBuf>,
) -> Result<CompiledProgram> {
    let source = if let Some(mcpp_inc_path) = mcpp_inc_path {
        // Try to find mcpp binary in the path
        let mcpp_path = which::which("mcpp").with_context(|| "mcpp binary not found")?;

        // Run mcpp to preprocess the program
        let mcpp_output = Command::new(&mcpp_path)
            .arg("-P")
            .arg("-I")
            .arg(mcpp_inc_path)
            .arg(source_path)
            .output()
            .with_context(|| format!("Failed to run mcpp: {}", mcpp_path.display()))?;

        if !mcpp_output.status.success() {
            anyhow::bail!("mcpp failed with exit code: {}", mcpp_output.status);
        }

        String::from_utf8_lossy(&mcpp_output.stdout).to_string()
    } else {
        fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read source file: {}", source_path.display()))?
    };

    let compiled = CompiledProgram::new(source, arguments, debug_symbols)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to compile program")?;

    Ok(compiled)
}

pub fn satisfy_program(
    compiled: CompiledProgram,
    witness: WitnessValues,
    prune: bool,
) -> Result<SatisfiedProgram> {
    if prune {
        println!("WARNING: program will be pruned using the provided witness, it might not work with a different one.");
        // TODO(m_kus): provide env via CLI
        let env = dummy_env::dummy();
        compiled
            .satisfy_with_env(witness, Some(&env))
            .map_err(|e| anyhow::anyhow!(e))
    } else {
        compiled.satisfy(witness).map_err(|e| anyhow::anyhow!(e))
    }
}

pub fn build_program(
    source_path: &Path,
    witness: Option<WitnessValues>,
    arguments: Option<Arguments>,
    prune: bool,
    debug_symbols: bool,
    mcpp_inc_path: Option<PathBuf>,
) -> Result<BuildArtifacts> {
    let compiled = compile_program(
        source_path,
        arguments.unwrap_or_default(),
        debug_symbols,
        mcpp_inc_path,
    )?;

    if let Some(witness) = witness {
        let satisfied = satisfy_program(compiled, witness, prune)?;
        let node = satisfied.redeem();
        println!("{}", format_node_bounds(&node.bounds()));

        let (program_bytes, witness_bytes) = node.encode_to_vec();

        let padding_size = node
            .bounds()
            .cost
            .get_padding(&vec![witness_bytes.clone(), program_bytes.clone()])
            .unwrap_or_default()
            .len();
        println!("Required padding size: {}", padding_size);

        Ok(BuildArtifacts {
            program: program_bytes,
            witness: Some(witness_bytes),
        })
    } else {
        Ok(BuildArtifacts {
            program: compiled.commit().encode_to_vec(),
            witness: None,
        })
    }
}

// TODO(m_kus): serialize debug symbols and allow to load artifacts instead of rebuilding for run
#[allow(dead_code)]
pub fn load_artifacts(target_dir: &PathBuf, program_name: &str) -> Result<BuildArtifacts> {
    let output_file = target_dir.join(program_name).with_extension("json");
    let json_content = fs::read_to_string(&output_file).with_context(|| {
        format!(
            "Failed to read build artifacts from {}",
            output_file.display()
        )
    })?;

    let artifacts: BuildArtifacts = serde_json::from_str(&json_content).with_context(|| {
        format!(
            "Failed to deserialize build artifacts from {}",
            output_file.display()
        )
    })?;

    Ok(artifacts)
}

pub fn build(args: BuildArgs) -> Result<()> {
    let witness = if let Some(witness_path) = args.witness {
        Some(load_witness(Some(&witness_path))?)
    } else {
        None
    };
    let artifacts = build_program(
        &args.entrypoint,
        witness,
        None,
        args.prune,
        false,
        args.mcpp_inc_path,
    )?;
    let program_name = get_program_name(&args.entrypoint)?;
    write_build_output(&args.target_dir, &program_name, artifacts, args.assembly)
}
