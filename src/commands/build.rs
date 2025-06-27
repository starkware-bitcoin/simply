use anyhow::{Context, Result};
use base64::display::Base64Display;
use base64::engine::general_purpose::STANDARD;
use clap::Args;
use simfony::{Arguments, CompiledProgram, WitnessValues};
use simplicity::human_encoding::Forest;
use simplicity::node::CommitNode;
use simplicity::{self, BitIter};
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct BuildArgs {
    /// Path to the source file
    pub path: PathBuf,

    /// Path to the witness file
    #[arg(long)]
    pub witness: Option<PathBuf>,

    /// Path to write the compiled program
    #[arg(long, name = "output-path")]
    pub output_path: Option<PathBuf>,

    /// Output format
    ///
    /// - base64: Base64 encoding
    /// - hex: Hex encoding
    /// - simpl: Disassembled Simplicity source code
    #[arg(long, name = "output-format", default_value = "base64")]
    pub output_format: String,
}

fn parse_witness(content: Option<&str>) -> Result<WitnessValues> {
    content.map_or(Ok(WitnessValues::default()), |s| {
        serde_json::from_str(s).with_context(|| "Failed to parse witness")
    })
}

fn write_build_output(
    output_path: Option<PathBuf>,
    program_bytes: Vec<u8>,
    output_format: String,
) -> Result<()> {
    let program_output = match output_format.as_str() {
        "hex" => format!("Program:\n{}", hex::encode(program_bytes)),
        "simpl" => {
            let iter = BitIter::from(program_bytes.into_iter());
            let commit = CommitNode::decode(iter)
                .map_err(|e| anyhow::anyhow!("failed to decode program: {}", e))?;
            let prog = Forest::<simplicity::jet::Elements>::from_program(commit);
            prog.string_serialize()
        }
        _ => format!(
            "Program:\n{}",
            Base64Display::new(&program_bytes, &STANDARD)
        ), // Default to base64
    };

    match output_path {
        Some(path) => {
            fs::write(&path, program_output)
                .with_context(|| format!("Failed to write output file: {}", path.display()))?;
        }
        None => {
            println!("{}", program_output);
        }
    }
    Ok(())
}

pub fn handle_build(args: BuildArgs) -> Result<()> {
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("Failed to read source file: {}", args.path.display()))?;

    let compiled = CompiledProgram::new(source, Arguments::default(), true)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to compile program")?;

    if let Some(witness_path) = args.witness {
        let witness_content = fs::read_to_string(&witness_path)
            .with_context(|| format!("Failed to read witness file: {}", witness_path.display()))?;

        let witness = parse_witness(Some(&witness_content))?;
        let satisfied = compiled.satisfy(witness).map_err(|e| anyhow::anyhow!(e))?;

        let node = satisfied.redeem();
        println!("Node bounds: {:?}", node.bounds());

        let (program_bytes, witness_bytes) = node.encode_to_vec();

        let padding_size = node
            .bounds()
            .cost
            .get_padding(&vec![witness_bytes, program_bytes.clone()])
            .unwrap_or_default()
            .len();
        println!("Padding size: {}", padding_size);

        write_build_output(args.output_path, program_bytes, args.output_format)
    } else {
        let program_bytes = compiled.commit().encode_to_vec();
        write_build_output(args.output_path, program_bytes, args.output_format)
    }
}
