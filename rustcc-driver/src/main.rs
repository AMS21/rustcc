use std::{path::PathBuf, process::Command};

fn main() {
    // Check if the user has passed any arguments
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        // If no arguments are passed, print the help message
        println!("Usage: rustcc-driver <source file>");
        return;
    }

    // Get the source file path
    let source_file_path = &args[0];

    // Get $CC environment variable or use "gcc" as default
    let cc = std::env::var("CC").unwrap_or("gcc".to_string());

    //let preprocessed_file_path = create_temp_file("preprocessed.i");
    let preprocessed_file_path = PathBuf::from(source_file_path).with_extension("i");

    println!("Preprocessing file '{}'...", source_file_path);

    // First preprocess the file using the C preprocessor
    Command::new(cc.clone())
        .arg("-E")
        .arg("-P")
        .arg(source_file_path)
        .arg("-o")
        .arg(&preprocessed_file_path)
        .status()
        .expect("Failed to preprocess the file!");

    println!(
        "Compiling file '{}'...",
        preprocessed_file_path.to_str().unwrap_or_default()
    );

    let assembly_file = PathBuf::from(source_file_path).with_extension("s");

    // Run the compiler
    Command::new(cc.clone())
        .arg("-S")
        .arg(&preprocessed_file_path)
        .arg("-o")
        .arg(&assembly_file)
        .status()
        .expect("Failed to compile the file!");

    // Delete the preprocessed file
    std::fs::remove_file(&preprocessed_file_path).expect("Failed to delete the preprocessed file!");

    println!(
        "Assembling and linking file '{}'...",
        assembly_file.to_str().unwrap_or_default(),
    );

    let output_file = PathBuf::from(source_file_path).with_extension("");

    // Assemble and link the file
    Command::new(cc)
        .arg(&assembly_file)
        .arg("-o")
        .arg(output_file)
        .status()
        .expect("Failed to assemble and link the file!");

    // Remove assembly file
    std::fs::remove_file(&assembly_file).expect("Failed to delete the assembly file!");
}
