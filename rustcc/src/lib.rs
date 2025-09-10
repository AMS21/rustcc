use std::{cell::RefCell, panic::PanicHookInfo, rc::Rc};

use codegen::Codegen;
use colored::Colorize;
use diagnostic_consumer::DefaultDiagnosticConsumer;
use diagnostic_engine::DiagnosticEngine;
use parser::Parser;
use source_manager::{RealFSSourceManager, SourceManager};

pub mod ast;
pub mod codegen;
pub mod command_line;
pub mod diagnostic;
pub mod diagnostic_builder;
pub mod diagnostic_consumer;
pub mod diagnostic_engine;
pub mod lexer;
pub mod parser;
pub mod source_file;
pub mod source_location;
pub mod source_manager;
pub mod source_range;
pub mod token;

pub fn run_main() {
    // Set a panic hook
    std::panic::set_hook(Box::new(panic_handler));

    // Handle command line arguments
    let command_line_matches = command_line::command_line().get_matches();

    // Get the first command line argument as the file path
    let file_path: &String = command_line_matches
        .get_one(command_line::ARG_INPUT_FILE)
        .unwrap();

    // Create our source manager
    let source_manager = RealFSSourceManager::new();

    // Create our diagnostic consumer
    let diagnostic_consumer = Box::new(DefaultDiagnosticConsumer);

    // Create our diagnostic engine
    let diagnostic_engine = Rc::new(RefCell::from(DiagnosticEngine::new(diagnostic_consumer)));

    // Load the input file into our source manager
    let source_file = source_manager.load_file(file_path.as_str()).map_or_else(
        || {
            eprintln!("Error reading file: '{file_path}'");
            // TODO: Once we recover the error handling, print the error message here
            //eprintln!("{error}");

            std::process::exit(1);
        },
        |source| source,
    );

    // Create a lexer
    let mut lexer = lexer::Lexer::new(diagnostic_engine.clone(), source_file);
    let tokens = lexer.tokenize();

    // Print all tokens
    if command_line_matches.get_flag(command_line::ARG_PRINT_TOKENS) {
        for token in &tokens {
            println!("{}", token.dump());
        }
    }

    // Create a parser
    let mut parser = Parser::new(diagnostic_engine.clone(), tokens);
    let translation_unit = parser.parse();

    // Print the abstract syntax tree (AST)
    if command_line_matches.get_flag(command_line::ARG_PRINT_AST) {
        println!("{}", translation_unit.dump());
    }

    // Codegen the translation unit
    let codegen = Codegen::new(file_path);

    codegen.codegen(&translation_unit);

    // Print the LLVM intermediate representation (IR)
    if command_line_matches.get_flag(command_line::ARG_PRINT_IR) {
        codegen.dump();
    }

    if diagnostic_engine.borrow().error_occurred() {
        std::process::exit(1);
    }
}

pub fn panic_handler(panic_info: &PanicHookInfo<'_>) {
    eprintln!(
        "{}\n",
        "Oh no rustcc encountered an internal error and has sadly crashed!"
            .bold()
            .red()
    );

    // Print location
    if let Some(location) = panic_info.location() {
        eprintln!(
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        eprintln!("Location: {}", "<unknown>".italic());
    }

    // Print panic message
    if let Some(string) = panic_info.payload().downcast_ref::<&str>() {
        eprintln!("Message: {}", string.yellow());
    } else if let Some(string) = panic_info.payload().downcast_ref::<String>() {
        eprintln!("Message: {}", string.yellow());
    } else {
        eprintln!("Message: {}", "<none>".italic());
    }

    // Print a backtrace
    eprintln!("Backtrace:");
    eprintln!("{}", std::backtrace::Backtrace::force_capture());

    // Print version and system info
    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
    eprintln!(
        "System:  {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    // Print command line
    let command_line_without_self = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    eprintln!(
        "\nCommand line: {}",
        if command_line_without_self.is_empty() {
            "<none>".italic().to_string()
        } else {
            command_line_without_self
        }
    );

    eprintln!(
        "\nPLEASE submit a bug report to {}",
        "https://github.com/AMS21/rustcc/issues".underline()
    );
}
