use anyhow::{Context, Result};
use simfony::{Arguments, WitnessValues};
use std::{fs, path::PathBuf};

/// Load witness from a JSON file.
pub fn load_witness(path: Option<&PathBuf>) -> Result<WitnessValues> {
    if let Some(path) = path {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read witness file: {}", path.display()))?;
        serde_json::from_str(&content).with_context(|| "Failed to parse witness")
    } else {
        Ok(WitnessValues::default())
    }
}

/// Load arguments from a JSON file.
pub fn load_arguments(path: Option<&PathBuf>) -> Result<Arguments> {
    if let Some(path) = path {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read arguments file: {}", path.display()))?;
        serde_json::from_str(&content).with_context(|| "Failed to parse arguments")
    } else {
        Ok(Arguments::default())
    }
}

/// Get program name from source path.
///
/// Program name is determined by the following rules:
/// 1. If file name is not main.simf, use it as program name (strip extension)
/// 2. If file is in <project dir>/src/<simf file>, use project name
/// 3. If file is in */<package name>/<simf file>, use package name
pub fn get_program_name(source_path: &PathBuf) -> Result<String> {
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
