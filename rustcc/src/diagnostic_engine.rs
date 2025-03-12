use crate::{diagnostic::Diagnostic, diagnostic_consumer::DiagnosticConsumer};

#[derive(Debug)]
pub struct DiagnosticEngine {
    number_of_warnings: u64,
    number_of_errors: u64,
    error_limit: u64,
    consumer: Box<dyn DiagnosticConsumer>,
    error_occurred: bool,
    fatal_error_occurred: bool,
    ignore_all_warnings: bool,
    warnings_as_errors: bool,
}

impl DiagnosticEngine {
    #[must_use]
    pub fn new(consumer: Box<dyn DiagnosticConsumer>) -> Self {
        Self {
            number_of_warnings: 0,
            number_of_errors: 0,
            error_limit: 0,
            consumer,
            error_occurred: false,
            fatal_error_occurred: false,
            ignore_all_warnings: false,
            warnings_as_errors: false,
        }
    }

    pub fn report(&mut self, diagnostic: &mut Diagnostic) {
        if self.ignore_all_warnings {
            diagnostic.ignore_warning();
        }

        if self.warnings_as_errors {
            diagnostic.upgrade_warning_to_error();
        }

        if diagnostic.is_error_or_fatal() {
            self.error_occurred = true;
            self.number_of_errors += 1;
        }

        if diagnostic.is_fatal_error() {
            self.fatal_error_occurred = true;
        }

        if diagnostic.is_warning() {
            self.number_of_warnings += 1;
        }

        self.consumer.report(diagnostic);
    }

    #[must_use]
    pub const fn error_occurred(&self) -> bool {
        self.error_occurred
    }

    #[must_use]
    pub const fn fatal_error_occurred(&self) -> bool {
        self.fatal_error_occurred
    }

    #[must_use]
    pub const fn error_limit_reached(&self) -> bool {
        self.error_limit > 0 && self.number_of_errors >= self.error_limit
    }
}
