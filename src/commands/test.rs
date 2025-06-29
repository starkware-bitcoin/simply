use anyhow::{Context, Result};
use clap::Args;
use regex::Regex;
use std::path::PathBuf;
use std::{fs, path::Path};
use walkdir::WalkDir;

use crate::commands::{run, BuildArgs, Logging, RunArgs};

// Colors for output
const GREEN: &str = "\x1b[0;32m";
const RED: &str = "\x1b[0;31m";
const NC: &str = "\x1b[0m"; // No Color

#[derive(Args)]
pub struct TestArgs {
    #[command(flatten)]
    pub build: BuildArgs,

    /// Print debug logs (info, debug, trace)
    #[arg(long, value_enum)]
    pub logging: Option<Logging>,
}

#[derive(Debug)]
struct TestResult {
    success: bool,
    error_message: Option<String>,
}

pub fn test(args: TestArgs) -> Result<()> {
    let mut test_results = Vec::new();
    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Find all *.simf files recursively in current directory
    let source_dir = args.build.path.parent().unwrap();
    let simf_files = find_simf_files(source_dir.to_str().unwrap())?;

    for file_path in simf_files {
        let test_functions = extract_test_functions(&file_path)?;

        for test_func in test_functions {
            total_tests += 1;
            let test_name = format!(
                "{}::{}",
                file_path.file_stem().unwrap().to_str().unwrap(),
                test_func
            );

            print!("{} ... ", test_name);

            let result = run_single_test(&file_path, &test_func, &args)?;

            if result.success {
                println!("{}ok{}", GREEN, NC);
                passed_tests += 1;
            } else {
                println!("{}err{}", RED, NC);
                if let Some(error) = &result.error_message {
                    eprintln!("{}", error);
                }
            }

            test_results.push(result);
        }
    }

    // Print summary
    let failed_tests = total_tests - passed_tests;
    if failed_tests > 0 {
        println!(
            "\ntest result: {}failed{}. {} passed; {} failed",
            RED, NC, passed_tests, failed_tests
        );
        std::process::exit(1);
    } else {
        println!(
            "\ntest result: {}success{}. {} passed; 0 failed",
            GREEN, NC, passed_tests
        );
    }

    Ok(())
}

fn find_simf_files(dir: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("simf") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn extract_test_functions(file_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let test_regex = Regex::new(r"fn (test_[a-zA-Z0-9_]*)").unwrap();
    let mut test_functions = Vec::new();

    for cap in test_regex.captures_iter(&content) {
        if let Some(test_func) = cap.get(1) {
            test_functions.push(test_func.as_str().to_string());
        }
    }

    Ok(test_functions)
}

fn run_single_test(file_path: &Path, test_func: &str, args: &TestArgs) -> Result<TestResult> {
    // Create temporary directory
    let temp_dir = tempfile::tempdir().with_context(|| "Failed to create temporary directory")?;

    let base_name = file_path.file_stem().unwrap().to_str().unwrap();
    let temp_file = temp_dir.path().join(format!("{}_temp.simf", base_name));

    // Read original file content
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Replace test function with main
    let modified_content = content.replace(&format!("fn {}", test_func), "fn main");

    // Write modified content to temp file
    fs::write(&temp_file, modified_content)
        .with_context(|| format!("Failed to write temp file: {}", temp_file.display()))?;

    // Create RunArgs for the test
    let mut run_args = RunArgs {
        build: args.build.clone(),
        param: None,
        logging: args.logging.clone(),
    };
    // Update the build path to use the temporary file
    run_args.build.path = temp_file;

    // Call run function directly
    match run(run_args) {
        Ok(_) => Ok(TestResult {
            success: true,
            error_message: None,
        }),
        Err(e) => Ok(TestResult {
            success: false,
            error_message: Some(format!("{}", e)),
        }),
    }
}
