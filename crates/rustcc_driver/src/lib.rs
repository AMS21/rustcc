use std::{
    cell::RefCell,
    env::consts::{ARCH, OS},
    panic::PanicHookInfo,
    process::ExitCode,
    rc::Rc,
};

use rustcc_codegen::Codegen;
use rustcc_diagnostic::{
    color::Colorize, diagnostic_consumer::DefaultDiagnosticConsumer,
    diagnostic_engine::DiagnosticEngine,
};
use rustcc_lexer::Lexer;
use rustcc_llvm::LLVM_VERSION;
use rustcc_parser::Parser;
use rustcc_source::source_manager::{RealFSSourceManager, SourceManager};

mod command_line;

pub fn main() -> ExitCode {
    // Set a panic hook
    std::panic::set_hook(Box::new(panic_handler));

    // Handle command line arguments
    let command_line_matches = command_line::command_line().get_matches();

    // Get the first command line argument as the file path
    let Some(file_path) = command_line_matches.get_one::<String>(command_line::ARG_INPUT_FILE)
    else {
        eprintln!("no input file");
        return ExitCode::FAILURE;
    };

    // Create our source manager
    let source_manager = RealFSSourceManager::new();

    // Create our diagnostic consumer
    let diagnostic_consumer = Box::new(DefaultDiagnosticConsumer);

    // Create our diagnostic engine
    let diagnostic_engine = Rc::new(RefCell::from(DiagnosticEngine::new(diagnostic_consumer)));

    // Load the input file into our source manager
    let Some(source_file) = source_manager.load_file(file_path.as_str()) else {
        eprintln!("Error reading file: '{file_path}'");
        // TODO: Once we recover the error handling, print the error message
        // here eprintln!("{error}");

        return ExitCode::FAILURE;
    };

    // Create a lexer
    let mut lexer = Lexer::new(diagnostic_engine.clone(), source_file);
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
    let Ok(codegen) = Codegen::new(file_path) else {
        eprintln!("Error initializing codegen");
        return ExitCode::FAILURE;
    };

    codegen.codegen(&translation_unit);

    // Print the LLVM intermediate representation (IR)
    if command_line_matches.get_flag(command_line::ARG_PRINT_IR) {
        codegen.dump();
    }

    if diagnostic_engine.borrow().error_occurred() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
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
        eprintln!("Message: {}", string.clone().yellow());
    } else {
        eprintln!("Message: {}", "<none>".italic());
    }

    // Print a backtrace
    eprintln!("Backtrace:");
    eprintln!("{}", std::backtrace::Backtrace::force_capture());

    // Print version and system info
    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
    eprintln!("LLVM:    {LLVM_VERSION}");
    eprintln!("System:  {OS}-{ARCH}");

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
