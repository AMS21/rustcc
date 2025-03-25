use assert_cmd::cargo::CommandCargoExt;
use clap::ArgAction;
use colored::Colorize;
use regex::RegexBuilder;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const ARG_DIRECTORY: &str = "DIRECTORY";
const ARG_UPDATE_BASELINE: &str = "UPDATE_BASELINE";

fn main() {
    let command_line = clap::Command::new(env!("CARGO_PKG_NAME"))
        .about("The test-driver for rustcc")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            clap::Arg::new(ARG_DIRECTORY)
                .short('d')
                .long("directory")
                .help("The tests directory")
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            clap::Arg::new(ARG_UPDATE_BASELINE)
                .short('u')
                .long("update-baseline")
                .help("update the expected output files instead of running tests")
                .action(ArgAction::SetTrue),
        )
        .arg_required_else_help(true);

    // Parse the command line arguments
    let matches = command_line.get_matches();

    // Extract arguments
    let directory: &String = matches.get_one(ARG_DIRECTORY).unwrap();
    let update_baseline = matches.get_flag(ARG_UPDATE_BASELINE);

    let input_dir = Path::new(&directory).join("input");
    let output_dir = Path::new(&directory).join("output");

    // Recursively find all `.c` files in the input directory
    let input_files = find_c_files(&input_dir);

    // Ensure that there are even any test files
    if input_files.is_empty() {
        println!("No test files found in '{}'", directory);
        process::exit(1);
    }

    let mut failed_tests = Vec::new();

    println!("Found {} test files in '{}'", input_files.len(), directory);

    let run_regex = RegexBuilder::new(r"^//\s*RUN:\s*(.*)$")
        .multi_line(true)
        .build()
        .expect("Failed to build regex");
    let binary_file_regex = RegexBuilder::new(r"\$\{\{(.+?)\}\}")
        .build()
        .expect("Failed to build regex");
    let expect_failure_regex = RegexBuilder::new(r"^//\s*EXPECT-FAILURE\s*$")
        .multi_line(true)
        .build()
        .expect("Failed to build regex");

    for input_path in &input_files {
        print!("Running test {}... ", input_path.display());

        // Construct the output path, preserving the directory structure
        let relative_path = input_path
            .strip_prefix(&input_dir)
            .expect("Failed to strip prefix");
        let output_path = output_dir.join(relative_path).with_extension("out");

        // Read the input file
        let input = fs::read_to_string(input_path).expect("Failed to read input file");

        // Extract run command from the input file
        let Some(run_command) = run_regex
            .captures(&input)
            .and_then(|captures| captures.get(1))
            .map(|m| m.as_str())
        else {
            println!("{}", "TEST ERROR".red());
            println!("Missing run directive");

            failed_tests.push(input_path);
            continue;
        };

        // Extract executable from the run command
        let Some(executable) = binary_file_regex
            .captures(run_command)
            .and_then(|capture| capture.get(1))
            .map(|m| m.as_str())
        else {
            println!("{}", "TEST ERROR".red());
            println!("Missing executable name in run directive");
            println!("Run directive: '{}'", run_command);

            failed_tests.push(input_path);
            continue;
        };

        // Remove executable from the run command
        let run_command = binary_file_regex.replace(run_command, "");

        // Collect the command line arguments
        let args = run_command.split_whitespace().collect::<Vec<_>>();

        // Check if the test is expected to fail
        let expect_failure = expect_failure_regex.is_match(&input);

        // Run executable on the input file
        let Ok(mut command) = process::Command::cargo_bin(executable) else {
            println!("{}", "TEST ERROR".red());
            println!("Executable '{}' not found", executable);

            failed_tests.push(input_path);
            continue;
        };

        let output = command
            .arg(input_path.to_str().unwrap())
            .args(args)
            .output()
            .expect("Failed to execute binary");

        // Extract status code
        let Some(status_code) = output.status.code() else {
            println!("{}", "TEST ERROR".red());
            println!("Failed to extract status code");
            continue;
        };

        // Check the status code
        if !expect_failure && status_code != 0 {
            println!("{}", "FAIL".red());
            println!("Test unexpectedly failed with status code: {status_code}");

            failed_tests.push(input_path);
            continue;
        } else if expect_failure && status_code == 0 {
            println!("{}", "FAIL".red());
            println!("Test unexpectedly passed");

            failed_tests.push(input_path);
            continue;
        }

        // Convert output to string
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        let output_str = format!("{}{}", stderr_str, stdout_str);

        if update_baseline {
            fs::create_dir_all(output_path.parent().unwrap())
                .expect("Failed to create output directory");
            fs::write(output_path, output_str).expect("Failed to write output file");
            println!("{}", "UPDATED".yellow());
        } else {
            // Read the expected output
            let Ok(expected_output) = fs::read_to_string(&output_path) else {
                println!("{}", "TEST ERROR".red());
                println!("Expected output file '{}' not found", output_path.display());

                failed_tests.push(input_path);
                continue;
            };

            // Compare the output
            if output_str.trim() == expected_output.trim() {
                println!("{}", "PASS".green());
            } else {
                println!("{}\n", "FAIL".red());
                println!("Expected:\n{}", expected_output);
                println!("Got:\n{}", output_str);

                failed_tests.push(input_path);
                continue;
            }
        }
    }

    if update_baseline {
        return;
    }

    // Print the summary
    println!("\nSummary:");
    println!(
        "Ran {} tests {} passed {} failed",
        input_files.len(),
        (input_files.len() - failed_tests.len()).to_string().green(),
        failed_tests.len().to_string().red()
    );

    // Print the failed tests
    if !failed_tests.is_empty() {
        println!("\nFailed tests:");
        for test in failed_tests {
            println!("{}", test.display());
        }

        // Exit with an error code
        process::exit(1);
    }
}

// Function to recursively find all `.c` files in a directory
fn find_c_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
                    files.push(path);
                }
            }
        }
    }

    files
}
