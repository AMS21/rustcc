#![no_main]

use std::{cell::RefCell, rc::Rc};

use libfuzzer_sys::{Corpus, fuzz_target};
use rustcc_codegen::Codegen;
use rustcc_diagnostic::{
    diagnostic_consumer::IgnoreDiagnosticConsumer, diagnostic_engine::DiagnosticEngine,
};
use rustcc_lexer::Lexer;
use rustcc_parser::Parser;
use rustcc_source::source_manager::{SourceManager, VirtualSourceManager};

const INPUT_FILE: &str = "fuzz.c";

// Reuse a single Codegen instance across fuzz iterations to avoid repeatedly
// creating and destroying the LLVM context and builder which are expensive.
// Use thread-local storage to avoid requiring Send/Sync for LLVM pointers.
thread_local! {
    static CODEGEN: RefCell<Codegen> = RefCell::new(
        Codegen::new(INPUT_FILE).expect("Failed to create codegen")
    );
}

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

    // If any errors occurred, reject this input and don't codegen
    if diagnostic_engine.borrow().error_occurred() {
        return Corpus::Keep;
    }

    // Codegen
    CODEGEN.with(|slot| {
        let mut codegen = slot.borrow_mut();
        // Create a fresh module inside the existing context
        let _ = codegen.reset_module(INPUT_FILE);

        assert!(codegen.codegen(&translation_unit));
    });

    Corpus::Keep
});
