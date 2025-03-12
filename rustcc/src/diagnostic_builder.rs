use std::{cell::RefCell, rc::Rc};

use crate::{
    diagnostic::{Diagnostic, DiagnosticNote},
    diagnostic_engine::DiagnosticEngine,
    source_range::SourceRange,
};

pub struct DiagnosticBuilder<'a> {
    engine: Rc<RefCell<DiagnosticEngine>>,
    diagnostic: Diagnostic<'a>,
}

impl<'a> DiagnosticBuilder<'a> {
    pub fn new(engine: Rc<RefCell<DiagnosticEngine>>, diagnostic: Diagnostic<'a>) -> Self {
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
