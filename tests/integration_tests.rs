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
    pub fn build(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("build")
            .arg("--entrypoint")
            .arg(source_path);

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

        let output_path = PathBuf::from("target").join(format!("{}.json", self.program_name));
        if !output_path.exists() {
            anyhow::bail!("Build output file not found: {}", output_path.display());
        }

        Ok(())
    }

    /// Run a simf file
    pub fn run(&self, source_path: &Path, witness_path: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("run")
            .arg("--entrypoint")
            .arg(source_path);

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
        cmd.arg("run")
            .arg("--")
            .arg("run")
            .arg("--logging")
            .arg("trace")
            .arg("--entrypoint")
            .arg(source_path);

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

        // Deposit the program
        println!("  Depositing...");
        self.deposit(source_path)?;
        println!("  ✓ Deposit successful");

        println!("✓ All tests passed for {}", self.program_name);
        Ok(())
    }

    pub fn deposit(&self, source_path: &Path) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("deposit")
            .arg("--entrypoint")
            .arg(source_path);

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute deposit command for {}",
                    self.program_name
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Deposit failed for {}: {}", self.program_name, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("tex1pvlydvg2lkew3068jyrsw469aac9lefnq0spjc7vqnxwrqrqmxrhsaythrt"));

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
