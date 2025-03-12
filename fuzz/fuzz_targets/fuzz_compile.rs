#![no_main]

use libfuzzer_sys::{fuzz_target, Corpus};
use rustcc::{diagnostic_consumer::IgnoreDiagnosticConsumer, diagnostic_engine::DiagnosticEngine};
use rustcc::{
    lexer::Lexer,
    source_manager::{SourceManager, VirtualSourceManager},
};
use std::cell::RefCell;
use std::rc::Rc;

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
    source_manager.add_file("fuzz.c", data);

    let Some(source_file) = source_manager.load_file("fuzz.c") else {
        return Corpus::Reject;
    };

    // Create a lexer
    let mut lexer = Lexer::new(diagnostic_engine, source_file);

    // Convert to tokens
    let _tokens = lexer.tokenize();

    Corpus::Keep
});
