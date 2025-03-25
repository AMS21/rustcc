#![no_main]

use libfuzzer_sys::{fuzz_target, Corpus};
use rustcc::{
    codegen::Codegen,
    diagnostic_consumer::IgnoreDiagnosticConsumer,
    diagnostic_engine::DiagnosticEngine,
    lexer::Lexer,
    parser::Parser,
    source_manager::{SourceManager, VirtualSourceManager},
};
use std::{cell::RefCell, rc::Rc};

const INPUT_FILE: &str = "fuzz.c";

fuzz_target!(|data: &[u8]| -> Corpus {
    // Convert input data to a string
    let Ok(data) = std::str::from_utf8(data) else {
        return Corpus::Reject;
    };

    let mut source_manager = VirtualSourceManager::new();

    // Create our diagnostic consumer
    let diagnostic_consumer = Box::new(IgnoreDiagnosticConsumer);

    // Create our diagnostic engine
    let diagnostic_engine = Rc::new(RefCell::from(DiagnosticEngine::new(diagnostic_consumer)));

    // Load the input file into our source manager
    source_manager.add_file(INPUT_FILE, data);

    let Some(source_file) = source_manager.load_file(INPUT_FILE) else {
        return Corpus::Reject;
    };

    // Tokenize
    let mut lexer = Lexer::new(diagnostic_engine.clone(), source_file);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = Parser::new(diagnostic_engine.clone(), tokens);
    let translation_unit = parser.parse();

    // Codegen
    let codegen = Codegen::new(INPUT_FILE);
    codegen.codegen(&translation_unit);

    Corpus::Keep
});
