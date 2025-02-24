use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use assert_cmd::cargo::CommandCargoExt;

#[test]
fn test_driver() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let input_dir = Path::new(&manifest_dir).join("tests/input");
    let output_dir = Path::new(&manifest_dir).join("tests/output");

    println!("Running tests...");
    println!("Input directory: {:?}", input_dir);
    println!("Output directory: {:?}", output_dir);

    // Recursively find all `.c` files in the input directory
    let input_files = find_c_files(&input_dir);

    for input_path in input_files {
        println!("Running test for: {:?}", input_path);

        // Construct the output path, preserving the directory structure
        let relative_path = input_path
            .strip_prefix(&input_dir)
            .expect("Failed to strip prefix");
        let output_path = output_dir.join(relative_path).with_extension("out");

        println!("Output path: {:?}", output_path);

        // Run rustcc on the input file
        let output = Command::cargo_bin("rustcc")
            .expect("Failed to find rustcc")
            .arg(input_path.to_str().unwrap())
            .output()
            .expect("Failed to execute rustcc");

        // Convert output to string
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Read the expected output
        let expected_output =
            fs::read_to_string(output_path).expect("Failed to read expected output");

        // Compare the output
        if output_str.trim() == expected_output.trim() {
            println!("Test passed for: {:?}", input_path);
        } else {
            println!("Test failed for: {:?}", input_path);
            println!("Expected:\n{}", expected_output);
            println!("Got:\n{}", output_str);
            panic!();
        }
    }
}

// Function to recursively find all `.c` files in a directory
fn find_c_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if dir.is_dir() {
        // Read the directory entries
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively search in subdirectories
                    files.extend(find_c_files(&path));
                } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
                    // Collect `.c` files
                    files.push(path);
                }
            }
        }
    }

    files
}
