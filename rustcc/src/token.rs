use std::collections::VecDeque;

use crate::source_range::SourceRange;

pub type TokenList<'a> = VecDeque<Token<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    KeywordInt,    // int
    KeywordReturn, // return
    KeywordVoid,   // void

    Identifier(String),

    // Literals
    IntegerLiteral(u32),

    // Symbols
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrace,        // {
    RightBrace,       // }
    Semicolon,        // ;
    Slash,            // /
    Tilde,            // ~
    Minus,            // -
    MinusMinus,       // --
}

impl TokenKind {
    #[must_use]
    pub fn from_identifier(identifier: &str) -> TokenKind {
        match identifier {
            "int" => TokenKind::KeywordInt,
            "return" => TokenKind::KeywordReturn,
            "void" => TokenKind::KeywordVoid,
            _ => TokenKind::Identifier(identifier.to_string()),
        }
    }

    #[must_use]
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::KeywordInt | TokenKind::KeywordReturn | TokenKind::KeywordVoid
        )
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Identifier(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: SourceRange<'a>,
}

impl<'a> Token<'a> {
    #[must_use]
    pub fn new(kind: TokenKind, range: SourceRange<'a>) -> Self {
        Self { kind, range }
    }

    #[must_use]
    pub fn new_identifier<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        Self {
            kind: TokenKind::from_identifier(range.source_text().unwrap()),
            range,
        }
    }

    #[must_use]
    pub fn new_integer_literal<R: Into<SourceRange<'a>>>(value: u32, range: R) -> Self {
        Self {
            kind: TokenKind::IntegerLiteral(value),
            range: range.into(),
        }
    }

    #[must_use]
    pub fn new_left_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "(");

        Self {
            kind: TokenKind::LeftParenthesis,
            range,
        }
    }

    #[must_use]
    pub fn new_right_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), ")");

        Self {
            kind: TokenKind::RightParenthesis,
            range,
        }
    }

    #[must_use]
    pub fn new_left_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "{");

        Self {
            kind: TokenKind::LeftBrace,
            range,
        }
    }

    #[must_use]
    pub fn new_right_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "}");

        Self {
            kind: TokenKind::RightBrace,
            range,
        }
    }

    #[must_use]
    pub fn new_semicolon<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), ";");

        Self {
            kind: TokenKind::Semicolon,
            range,
        }
    }

    #[must_use]
    pub fn new_slash<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "/");

        Self {
            kind: TokenKind::Slash,
            range,
        }
    }

    #[must_use]
    pub fn new_tilde<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "~");

        Self {
            kind: TokenKind::Tilde,
            range,
        }
    }

    #[must_use]
    pub fn new_minus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "-");

        Self {
            kind: TokenKind::Minus,
            range,
        }
    }

    #[must_use]
    pub fn new_minus_minus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "--");

        Self {
            kind: TokenKind::MinusMinus,
            range,
        }
    }

    #[must_use]
    pub fn is_keyword(&self) -> bool {
        self.kind.is_keyword()
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    #[must_use]
    pub fn source_text(&self) -> Option<&'a str> {
        self.range.source_text()
    }

    #[must_use]
    pub fn dump(&self) -> String {
        if self.range.begin == self.range.end {
            let location = self.range.begin;
            return format!(
                "{:?} {}:{} - '{}'",
                self.kind,
                location.line,
                location.column,
                self.source_text().unwrap_or_default()
            );
        }

        format!(
            "{:?} {}:{}-{}:{} - '{}'",
            self.kind,
            self.range.begin.line,
            self.range.begin.column,
            self.range.end.line,
            self.range.end.column,
            self.source_text().unwrap_or_default()
        )
    }
}
