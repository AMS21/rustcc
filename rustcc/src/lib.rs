use std::{cell::RefCell, rc::Rc};

use diagnostic_consumer::DefaultDiagnosticConsumer;
use diagnostic_engine::DiagnosticEngine;
use source_manager::{RealFSSourceManager, SourceManager};

pub mod ast;
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
    let source_file = match source_manager.load_file(file_path.as_str()) {
        Some(source) => source,
        None => {
            eprintln!("Error reading file: '{file_path}'");
            // TODO: Once we recover the error handling, print the error message here
            //eprintln!("{error}");

            std::process::exit(1);
        }
    };

    // Create a lexer
    let mut lexer = lexer::Lexer::new(diagnostic_engine.clone(), source_file);

    // Print all tokens
    if command_line_matches.get_flag(command_line::ARG_PRINT_TOKENS) {
        loop {
            let token = lexer.next_token();
            println!("{}", token.dump());

            if token.is_eof() {
                break;
            }
        }
    }

    if diagnostic_engine.borrow().error_occurred() {
        std::process::exit(1);
    }
}
