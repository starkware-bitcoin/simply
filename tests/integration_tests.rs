use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Generic test runner for simf files
pub struct SimfTestRunner {
    program_name: String,
}

impl SimfTestRunner {
    /// Create a new test runner for a simf file
    pub fn new(program_name: &str) -> Self {
        Self {
            program_name: program_name.to_string(),
        }
    }

    /// Build a simf file
    pub fn build(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<Vec<u8>> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("build")
            .arg(source_path)
            .arg("--output-format")
            .arg("hex");

        if let Some(witness_path) = witness_path {
            cmd.arg("--witness").arg(witness_path);
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| {
                format!("Failed to execute build command for {}", self.program_name)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Build failed for {}: {}", self.program_name, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let hex_output = stdout
            .lines()
            .find(|line| line.starts_with("Program:"))
            .ok_or_else(|| anyhow::anyhow!("No program output found in build result"))?
            .trim_start_matches("Program:")
            .trim();

        let program_bytes =
            hex::decode(hex_output).with_context(|| "Failed to decode hex program output")?;

        Ok(program_bytes)
    }

    /// Run a simf file
    pub fn run(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run").arg("--").arg("run").arg(source_path);

        if let Some(witness_path) = witness_path {
            cmd.arg("--witness").arg(witness_path);
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute run command for {}", self.program_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Run failed for {}: {}", self.program_name, stderr);
        }

        Ok(())
    }

    /// Debug a simf file
    pub fn debug(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run").arg("--").arg("debug").arg(source_path);

        if let Some(witness_path) = witness_path {
            cmd.arg("--witness").arg(witness_path);
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| {
                format!("Failed to execute debug command for {}", self.program_name)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Debug failed for {}: {}", self.program_name, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Run a complete test: build, run, and debug
    pub fn test(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<()> {
        println!("Testing {}...", self.program_name);

        // Build the program
        println!("  Building...");
        let _program_bytes = self.build(source_path, witness_path)?;
        println!("  ✓ Build successful");

        // Run the program
        println!("  Running...");
        self.run(source_path, witness_path)?;
        println!("  ✓ Run successful");

        // Debug the program
        println!("  Debugging...");
        let debug_output = self.debug(source_path, witness_path)?;
        println!("  ✓ Debug successful");
        println!("  Debug output length: {} characters", debug_output.len());

        println!("✓ All tests passed for {}", self.program_name);
        Ok(())
    }
}

#[test]
fn test_sighash_none() -> Result<()> {
    let runner = SimfTestRunner::new("sighash_none");

    let source_path = PathBuf::from("tests/data/sighash_none.simf");
    let witness_path = PathBuf::from("tests/data/sighash_none.wit");

    runner.test(&source_path, Some(&witness_path))?;

    Ok(())
}
