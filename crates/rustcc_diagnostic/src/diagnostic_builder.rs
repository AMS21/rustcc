use std::{cell::RefCell, rc::Rc};

use rustcc_source::source_range::SourceRange;

use crate::{
    diagnostic::Diagnostic, diagnostic_engine::DiagnosticEngine, diagnostic_note::DiagnosticNote,
};

#[derive(Debug, Clone)]
pub struct DiagnosticBuilder<'a> {
    engine: Rc<RefCell<DiagnosticEngine>>,
    diagnostic: Diagnostic<'a>,
}

impl<'a> DiagnosticBuilder<'a> {
    pub const fn new(engine: Rc<RefCell<DiagnosticEngine>>, diagnostic: Diagnostic<'a>) -> Self {
        Self { engine, diagnostic }
    }

    pub fn add_note<S: Into<String>, R: Into<SourceRange<'a>>>(
        &mut self,
        source_range: R,
        message: S,
    ) {
        self.diagnostic.add_note(DiagnosticNote {
            message: message.into(),
            source_range: source_range.into(),
        });
    }
}

impl Drop for DiagnosticBuilder<'_> {
    fn drop(&mut self) {
        self.engine.borrow_mut().report(&mut self.diagnostic);
    }
}
