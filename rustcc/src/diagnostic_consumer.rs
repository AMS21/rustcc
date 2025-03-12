use colored::Colorize;

use crate::diagnostic::{Diagnostic, DiagnosticLevel};

use std::fmt::Debug;

pub trait DiagnosticConsumer: Debug {
    fn report(&self, diagnostic: &Diagnostic);
}

// -- Ignore Diagnostic Consumer --

/// A diagnostic consumer which simply ignores the diagnostics
#[derive(Default, Debug)]
pub struct IgnoreDiagnosticConsumer;

impl DiagnosticConsumer for IgnoreDiagnosticConsumer {
    fn report(&self, _diagnostic: &Diagnostic) {}
}

// -- Default Diagnostic Consumer --

#[derive(Default, Debug)]
pub struct DefaultDiagnosticConsumer;

/// The default consumer prints all warnings to stdout and errors to stderr
impl DiagnosticConsumer for DefaultDiagnosticConsumer {
    fn report(&self, diagnostic: &Diagnostic) {
        debug_assert!(
            !diagnostic.is_ignored(),
            "May not report ignored diagnostics"
        );
        debug_assert!(
            !diagnostic.message.is_empty(),
            "May not report empty messages"
        );

        let begin_location = &diagnostic.source_range.begin.to_string().bold();
        let message = &diagnostic.message;

        match diagnostic.level {
            DiagnosticLevel::Warning => {
                println!("{begin_location}: {} {message}", "warning:".yellow())
            }
            DiagnosticLevel::Error => {
                eprintln!("{begin_location}: {} {message}", "error:".red().bold())
            }
            DiagnosticLevel::FatalError => {
                eprintln!(
                    "{begin_location}: {} {message}",
                    "fatal error:".red().bold()
                )
            }
            DiagnosticLevel::Ignored => {
                unreachable!("Unexpected diagnostic level");
            }
        }

        // Print any associated notes
        for note in &diagnostic.notes {
            let note_begin_location = &note.source_range.begin;
            let note_message = &note.message;

            println!("{note_begin_location}: note: {note_message}");
        }
    }
}
