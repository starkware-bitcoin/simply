use anyhow::{Context, Result};
use clap::Args;
use simfony::{Arguments, CompiledProgram, WitnessValues};
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct BuildArgs {
    /// Path to the source file
    pub path: PathBuf,

    /// Path to the witness file
    #[arg(long)]
    pub witness: Option<PathBuf>,

    /// Name of the program (will use project name by default)
    #[arg(long)]
    pub program_name: Option<String>,

    /// Output directory for the compiled program (will use `target` by default)
    #[arg(long, name = "target-dir", default_value = "./target")]
    pub target_dir: PathBuf,
}

fn parse_witness(content: Option<&str>) -> Result<WitnessValues> {
    content.map_or(Ok(WitnessValues::default()), |s| {
        serde_json::from_str(s).with_context(|| "Failed to parse witness")
    })
}

fn format_node_bounds(bounds: &simplicity::NodeBounds) -> String {
    format!(
        "Node bounds:\n  Extra cells: {}\n  Extra frames: {}\n  CPU cost: {}",
        bounds.extra_cells, bounds.extra_frames, bounds.cost
    )
}

fn get_program_name(source_path: &PathBuf) -> Result<String> {
    let canonical_path = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.clone());
    let components: Vec<_> = canonical_path.components().collect();

    // Rule 0: if file name is not main.simf, use it as program name (strip extension)
    if let Some(file_name) = source_path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            if file_name_str != "main.simf" {
                return Ok(file_name_str.to_string().replace(".simf", ""));
            }
        }
    }

    // Rule 1: if file is in <project dir>/src/<simf file>
    // Check if path has at least 3 components and second-to-last is "src"
    if components.len() >= 3 {
        if let std::path::Component::Normal(name) = components[components.len() - 2] {
            if name.to_str() == Some("src") {
                if let std::path::Component::Normal(project_name) = components[components.len() - 3]
                {
                    if let Some(name_str) = project_name.to_str() {
                        return Ok(name_str.to_string());
                    }
                }
            }
        }
    }

    // Rule 2: if file is in */<package name>/<simf file>
    // Check if path has at least 2 components
    if components.len() >= 2 {
        if let std::path::Component::Normal(package_name) = components[components.len() - 2] {
            if let Some(name_str) = package_name.to_str() {
                return Ok(name_str.to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "Cannot determine project name from path: {}",
        source_path.display()
    ))
}

fn write_build_output(
    target_dir: &PathBuf,
    program_name: &str,
    program_bytes: Vec<u8>,
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
    let output_file = target_dir.join(format!("{}.bin", program_name));

    // Write binary data directly to file
    fs::write(&output_file, program_bytes)
        .with_context(|| format!("Failed to write output file: {}", output_file.display()))?;

    println!("Compiled program written to: {}", output_file.display());

    Ok(())
}

pub fn handle_build(args: BuildArgs) -> Result<()> {
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("Failed to read source file: {}", args.path.display()))?;

    let compiled = CompiledProgram::new(source, Arguments::default(), true)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| "Failed to compile program")?;

    let program_bytes = if let Some(witness_path) = args.witness {
        println!("WARNING: program will be pruned using the provided witness, it might not work with a different one.");

        let witness_content = fs::read_to_string(&witness_path)
            .with_context(|| format!("Failed to read witness file: {}", witness_path.display()))?;

        let witness = parse_witness(Some(&witness_content))?;
        let satisfied = compiled.satisfy(witness).map_err(|e| anyhow::anyhow!(e))?;

        let node = satisfied.redeem();
        println!("{}", format_node_bounds(&node.bounds()));

        let (program_bytes, witness_bytes) = node.encode_to_vec();

        let padding_size = node
            .bounds()
            .cost
            .get_padding(&vec![witness_bytes, program_bytes.clone()])
            .unwrap_or_default()
            .len();
        println!("Padding size: {}", padding_size);

        program_bytes
    } else {
        compiled.commit().encode_to_vec()
    };

    let program_name = if let Some(program_name) = args.program_name {
        program_name
    } else {
        get_program_name(&args.path)?
    };

    write_build_output(&args.target_dir, &program_name, program_bytes)
}
