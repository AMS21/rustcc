use crate::source_range::SourceRange;
use DiagnosticLevel::{Error, Warning};

macro_rules! define_diagnostics {
    ($(
        $name:ident($level:expr, $flag:expr),
    )*) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum DiagnosticId {
            $(
                $name,
            )*
        }

        impl DiagnosticId {
            #[must_use]
            pub const fn level(&self) -> DiagnosticLevel {
                match self {
                    $(
                        DiagnosticId::$name => $level,
                    )*
                }
            }

            #[must_use]
            pub const fn flag_name(&self) -> &'static str {
                match self {
                    $(
                        DiagnosticId::$name => $flag,
                    )*
                }
            }
        }
    };
}

define_diagnostics! {
    // Lexer warnings
    NullCharacter(Warning, "-Wnull-character"),

    // Lexer errors
    UnexpectedCharacter(Error, ""),
    IntegerLiteralTooLarge(Error, ""),

    // Lexer fatal errors

    // Parser warnings

    // Parser errors
    ExpectedFunctionReturnType(Error, ""),
    ExpectedFunctionName(Error, ""),
    ExpectedLeftParenthesis(Error, ""),
    ExpectedRightParenthesis(Error, ""),
    ExpectedLeftBrace(Error, ""),
    ExpectedRightBrace(Error, ""),
    ExpectedSemicolon(Error, ""),
    ExpectedReturnKeyword(Error, ""),
    ExpectedIntegerLiteral(Error, ""),
    ExpectedVoidInParameterList(Error, ""),
    ExpectedExpression(Error, ""),

    // Parser fatal errors
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Ignored,
    Warning,
    Error,
    FatalError,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic<'a> {
    pub id: DiagnosticId,
    pub level: DiagnosticLevel,
    pub source_range: SourceRange<'a>,
    pub message: String,
    pub notes: Vec<DiagnosticNote<'a>>,
}

impl<'a> Diagnostic<'a> {
    #[must_use]
    pub fn new<R: Into<SourceRange<'a>>, S: Into<String>>(
        id: DiagnosticId,
        source_range: R,
        message: S,
    ) -> Self {
        Self {
            id,
            level: id.level(),
            source_range: source_range.into(),
            message: message.into(),
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub const fn is_ignored(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Ignored)
    }

    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Warning)
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Error)
    }

    #[must_use]
    pub const fn is_fatal_error(&self) -> bool {
        matches!(self.level, DiagnosticLevel::FatalError)
    }

    #[must_use]
    pub const fn is_error_or_fatal(&self) -> bool {
        self.is_error() || self.is_fatal_error()
    }

    pub fn upgrade_warning_to_error(&mut self) {
        if self.level == DiagnosticLevel::Warning {
            self.level = DiagnosticLevel::Error;
        }
    }

    pub fn ignore_warning(&mut self) {
        if self.level == DiagnosticLevel::Warning {
            self.level = DiagnosticLevel::Ignored;
        }
    }

    pub fn add_note(&mut self, note: DiagnosticNote<'a>) {
        self.notes.push(note);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticNote<'a> {
    pub source_range: SourceRange<'a>,
    pub message: String,
}
